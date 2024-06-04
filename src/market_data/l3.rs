use super::{BidOffer, MarketSide, UpdateAction};
use std::{
    collections::BTreeMap,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

struct MarketLiquidity<A> {
    size: A,
}

struct MarketLevel<I, A> {
    size: A,
    prices: BTreeMap<I, MarketLiquidity<A>>,
}

struct MarketLiquidityMap<P> {
    side: MarketSide,
    price: P,
}

pub struct L3MarketData<I, P, A>
where
    I: Ord + Copy,
    P: Ord + Copy + Add<Output = P> + Div<Output = P> + From<i32> + Mul<A, Output = A>,
    A: Default
        + PartialOrd
        + AddAssign
        + SubAssign
        + Copy
        + Sub<Output = A>
        + Add<Output = A>
        + Div<A, Output = P>,
{
    bids: BTreeMap<P, MarketLevel<I, A>>,
    offers: BTreeMap<P, MarketLevel<I, A>>,
    prices: BTreeMap<I, MarketLiquidityMap<P>>,
}

impl<I, P, A> L3MarketData<I, P, A>
where
    I: Ord + Copy,
    P: Ord + Copy + Add<Output = P> + Div<Output = P> + From<i32> + Mul<A, Output = A>,
    A: Default
        + PartialOrd
        + AddAssign
        + SubAssign
        + Copy
        + Sub<Output = A>
        + Add<Output = A>
        + Div<A, Output = P>,
{
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            offers: BTreeMap::new(),
            prices: BTreeMap::new(),
        }
    }

    pub fn update(
        &mut self,
        action: UpdateAction,
        side: MarketSide,
        id: I,
        price: P,
        size: A,
    ) -> Result<(), ()> {
        match action {
            UpdateAction::Add => {
                let side_store = match side {
                    MarketSide::Bid => &mut self.bids,
                    MarketSide::Offer => &mut self.offers,
                };

                Self::add_price(side_store, id, price, size);
                self.prices.insert(id, MarketLiquidityMap { side, price });

                Ok(())
            }
            UpdateAction::Update => {
                if let Some(liquidity_map) = self.prices.get_mut(&id) {
                    let side_store = match liquidity_map.side {
                        MarketSide::Bid => &mut self.bids,
                        MarketSide::Offer => &mut self.offers,
                    };

                    let mut replace_price = false;

                    if let Some(level) = side_store.get_mut(&liquidity_map.price) {
                        if liquidity_map.price == price {
                            if let Some(liquidity) = level.prices.get_mut(&id) {
                                level.size += size - liquidity.size;
                                liquidity.size = size;
                            } else {
                                return Err(());
                            }
                        } else {
                            replace_price = true;
                        }
                    } else {
                        return Err(());
                    }

                    if replace_price {
                        Self::remove_price(side_store, id, liquidity_map.price);
                        Self::add_price(side_store, id, price, size);
                        liquidity_map.price = price;
                    }

                    Ok(())
                } else {
                    Err(())
                }
            }
            UpdateAction::Remove => {
                if let Some(liquidity_map) = self.prices.remove(&id) {
                    let side_store = match liquidity_map.side {
                        MarketSide::Bid => &mut self.bids,
                        MarketSide::Offer => &mut self.offers,
                    };

                    Self::remove_price(side_store, id, liquidity_map.price);

                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }

    fn add_price(side_store: &mut BTreeMap<P, MarketLevel<I, A>>, id: I, price: P, size: A) {
        let entry = side_store.entry(price).or_insert(MarketLevel {
            size: A::default(),
            prices: BTreeMap::new(),
        });

        entry.prices.insert(id, MarketLiquidity { size });
        entry.size += size;
    }

    fn remove_price(side_store: &mut BTreeMap<P, MarketLevel<I, A>>, id: I, price: P) {
        let mut remove_level = false;
        if let Some(level) = side_store.get_mut(&price) {
            if let Some(liquidity) = level.prices.remove(&id) {
                if level.prices.is_empty() {
                    remove_level = true;
                } else {
                    level.size -= liquidity.size;
                }
            }
        }
        if remove_level {
            side_store.remove(&price);
        }
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.offers.clear();
        self.prices.clear();
    }

    pub fn get_price(&self, size: A) -> BidOffer<P> {
        BidOffer::new_with_price(
            self.calc_vwap(size, self.bids.iter().rev()),
            self.calc_vwap(size, self.offers.iter()),
        )
    }

    fn calc_vwap<'a, T>(&self, size: A, iter: T) -> Option<P>
    where
        T: Iterator<Item = (&'a P, &'a MarketLevel<I, A>)>,
        I: 'a,
        A: 'a,
        P: 'a,
    {
        let mut current_size = A::default();
        let mut current_total = A::default();

        for (&next_price, next_size) in iter {
            let mut incremental_size = next_size.size;

            if next_size.size + current_size > size {
                incremental_size = size - current_size;
            }

            current_total += next_price * incremental_size;
            current_size += incremental_size;

            // Should always be equal at this point but just in case of some weird rounding issues
            if current_size >= size {
                return Some(current_total / current_size);
            }
        }

        None
    }
}

impl<I, P, A> Default for L3MarketData<I, P, A>
where
    I: Ord + Copy,
    P: Ord + Copy + Add<Output = P> + Div<Output = P> + From<i32> + Mul<A, Output = A>,
    A: Default
        + PartialOrd
        + AddAssign
        + SubAssign
        + Copy
        + Sub<Output = A>
        + Add<Output = A>
        + Div<A, Output = P>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_add_price() {
        let mut test = L3MarketData::new();

        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Bid, 123, 12, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Offer, 124, 15, 20),
            Ok(())
        );

        assert_eq!(
            test.get_price(1),
            BidOffer::new_with_price(Some(12), Some(15))
        );
        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(12), Some(15))
        );

        assert_eq!(test.get_price(11), BidOffer::new_with_price(None, Some(15)));
        assert_eq!(test.get_price(20), BidOffer::new_with_price(None, Some(15)));
        assert_eq!(test.get_price(21), BidOffer::new_with_price(None, None));
    }

    #[test]
    fn simple_modify_size() {
        let mut test = L3MarketData::new();

        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Bid, 123, 12, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Offer, 124, 15, 20),
            Ok(())
        );

        assert_eq!(
            test.update(UpdateAction::Update, MarketSide::Bid, 123, 12, 8),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Update, MarketSide::Offer, 124, 15, 12),
            Ok(())
        );

        assert_eq!(
            test.get_price(1),
            BidOffer::new_with_price(Some(12), Some(15))
        );
        assert_eq!(
            test.get_price(8),
            BidOffer::new_with_price(Some(12), Some(15))
        );
        assert_eq!(test.get_price(9), BidOffer::new_with_price(None, Some(15)));
        assert_eq!(test.get_price(12), BidOffer::new_with_price(None, Some(15)));
        assert_eq!(test.get_price(13), BidOffer::new_with_price(None, None));
    }

    #[test]
    fn simple_modify_price() {
        let mut test = L3MarketData::new();

        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Bid, 123, 12, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Offer, 124, 15, 20),
            Ok(())
        );

        assert_eq!(
            test.update(UpdateAction::Update, MarketSide::Bid, 123, 11, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Update, MarketSide::Offer, 124, 16, 20),
            Ok(())
        );

        assert_eq!(
            test.get_price(1),
            BidOffer::new_with_price(Some(11), Some(16))
        );
        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(11), Some(16))
        );
        assert_eq!(test.get_price(11), BidOffer::new_with_price(None, Some(16)));
        assert_eq!(test.get_price(20), BidOffer::new_with_price(None, Some(16)));
        assert_eq!(test.get_price(21), BidOffer::new_with_price(None, None));
    }

    #[test]
    fn simple_remove_price() {
        let mut test = L3MarketData::new();

        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Bid, 123, 12, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Offer, 124, 15, 20),
            Ok(())
        );

        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(12), Some(15))
        );

        assert_eq!(
            test.update(UpdateAction::Remove, MarketSide::Bid, 123, 11, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Remove, MarketSide::Offer, 124, 16, 20),
            Ok(())
        );

        assert_eq!(test.get_price(10), BidOffer::new_with_price(None, None));
    }

    #[test]
    fn multi_price_on_level() {
        let mut test = L3MarketData::new();

        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Bid, 123, 12, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Offer, 124, 15, 10),
            Ok(())
        );

        assert_eq!(test.get_price(12), BidOffer::new_with_price(None, None));

        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Bid, 125, 12, 5),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Offer, 126, 15, 5),
            Ok(())
        );

        assert_eq!(
            test.get_price(12),
            BidOffer::new_with_price(Some(12), Some(15))
        );

        assert_eq!(
            test.update(UpdateAction::Remove, MarketSide::Bid, 123, 11, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Remove, MarketSide::Offer, 124, 16, 10),
            Ok(())
        );

        assert_eq!(
            test.get_price(5),
            BidOffer::new_with_price(Some(12), Some(15))
        );
        assert_eq!(test.get_price(10), BidOffer::new_with_price(None, None));
    }

    #[test]
    fn multi_price_modify() {
        let mut test = L3MarketData::new();

        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Bid, 123, 12, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Offer, 124, 15, 10),
            Ok(())
        );

        assert_eq!(test.get_price(12), BidOffer::new_with_price(None, None));

        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Bid, 125, 12, 5),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Add, MarketSide::Offer, 126, 15, 5),
            Ok(())
        );

        assert_eq!(
            test.get_price(12),
            BidOffer::new_with_price(Some(12), Some(15))
        );

        assert_eq!(
            test.update(UpdateAction::Update, MarketSide::Bid, 123, 12, 15),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Update, MarketSide::Offer, 124, 15, 15),
            Ok(())
        );

        assert_eq!(
            test.get_price(20),
            BidOffer::new_with_price(Some(12), Some(15))
        );

        assert_eq!(
            test.update(UpdateAction::Update, MarketSide::Bid, 123, 10, 10),
            Ok(())
        );
        assert_eq!(
            test.update(UpdateAction::Update, MarketSide::Offer, 124, 17, 10),
            Ok(())
        );

        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(11), Some(16))
        );
    }
}
