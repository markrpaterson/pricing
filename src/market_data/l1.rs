use std::{
    cell::RefCell,
    ops::{Add, Div},
};

use super::BidOffer;

pub trait MarketCallback {
    fn market_updated(&self);
}

/// A structure to hold L1 pricing, i.e. a single level of pricing.  This can either be to be the top of book of a deeper
/// market, or for pricing a Fixed Income instrument which tends to be priced this way.  There are also fields which can be used
/// to define the largest amount this price is valid for.
///
/// # Generic Parameters
///
/// * `A` - The amount type that should be used.
/// * `P` - The price type that should be used.
pub struct L1MarketData<'a, A, P>
where
    A: Copy + PartialOrd + Add<Output = A> + Div<Output = A> + From<i32>,
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
{
    price: BidOffer<P>,
    max: BidOffer<A>,

    callbacks: RefCell<Vec<&'a dyn MarketCallback>>,
}

impl<'a, A, P> L1MarketData<'a, A, P>
where
    A: Copy + PartialOrd + Add<Output = A> + Div<Output = A> + From<i32>,
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
{
    /// Use the new function to create a new L1MarketData with a bid and offer price.
    ///
    /// # Parameters
    ///
    /// * `bid` - The bid price, a value of None means no price available
    /// * `offer` - The offer price, a value of None means no price available
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let market_data = L1MarketData::<i32, i32>::new(Some(10), Some(20));
    /// ```
    pub fn new(bid: Option<P>, offer: Option<P>) -> Self {
        L1MarketData::new_with_max(bid, offer, None, None)
    }

    /// Use the new function to create a new L1MarketData with a bid and offer price as well as maximum sizes.
    ///
    /// # Parameters
    ///
    /// * `bid` - The bid price, a value of None means no price available
    /// * `offer` - The offer price, a value of None means no price available
    /// * `max_bid` - The max size that the bid price is valid.  A value of None means there is no limit.
    /// * `max_offer` - The max size that the offer price is valid.  A value of None means there is no limit.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let market_data = L1MarketData::new_with_max(Some(10), Some(20), Some(100), Some(100));
    /// ```
    pub fn new_with_max(
        bid: Option<P>,
        offer: Option<P>,
        max_bid: Option<A>,
        max_offer: Option<A>,
    ) -> Self {
        Self {
            price: BidOffer::new(bid, offer),
            max: BidOffer::new(max_bid, max_offer),
            callbacks: RefCell::new(Vec::new()),
        }
    }

    /// Get the current bid price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let market_data = L1MarketData::<i32, i32>::new(Some(10), Some(20));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    /// ```
    pub fn get_bid(&self) -> &Option<P> {
        self.price.get_bid()
    }

    /// Get the current offer price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let market_data = L1MarketData::<i32, i32>::new(Some(10), Some(20));
    ///
    /// assert_eq!(*market_data.get_offer(), Some(20));
    /// ```
    pub fn get_offer(&self) -> &Option<P> {
        self.price.get_offer()
    }

    /// Get the current mid price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let market_data = L1MarketData::<i32, i32>::new(Some(10), Some(20));
    ///
    /// assert_eq!(market_data.get_mid(), Some(15));
    /// ```
    pub fn get_mid(&self) -> Option<P> {
        self.price.get_mid()
    }

    /// Get the maximum size the bid is valid for
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let market_data = L1MarketData::new_with_max(Some(10), Some(20), Some(90), Some(110));
    ///
    /// assert_eq!(*market_data.get_max_bid(), Some(90));
    /// ```
    pub fn get_max_bid(&self) -> &Option<A> {
        self.max.get_bid()
    }

    /// Get the maximum size the offer is valid for
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let market_data = L1MarketData::new_with_max(Some(10), Some(20), Some(90), Some(110));
    ///
    /// assert_eq!(*market_data.get_max_offer(), Some(110));
    /// ```
    pub fn get_max_offer(&self) -> &Option<A> {
        self.max.get_offer()
    }

    /// Update the bid price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let mut market_data = L1MarketData::<i32, i32>::new(Some(10), Some(20));
    ///
    /// market_data.update_bid(Some(12));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(12));
    /// ```
    pub fn update_bid(&mut self, bid: Option<P>) {
        if *self.price.get_bid() != bid {
            self.price = BidOffer::new(bid, *self.price.get_offer());
            self.publish_to_subscribers();
        }
    }

    /// Update the offer price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let mut market_data = L1MarketData::<i32, i32>::new(Some(10), Some(20));
    ///
    /// market_data.update_offer(Some(22));
    ///
    /// assert_eq!(*market_data.get_offer(), Some(22));
    /// ```
    pub fn update_offer(&mut self, offer: Option<P>) {
        if *self.price.get_offer() != offer {
            self.price = BidOffer::new(*self.price.get_bid(), offer);
            self.publish_to_subscribers();
        }
    }

    pub fn update_max_bid(&mut self, max_bid: Option<A>) {
        if *self.max.get_bid() != max_bid {
            self.max = BidOffer::new(max_bid, *self.max.get_offer());
            self.publish_to_subscribers();
        }
    }

    pub fn update_max_offer(&mut self, max_offer: Option<A>) {
        if *self.max.get_offer() != max_offer {
            self.max = BidOffer::new(*self.max.get_bid(), max_offer);
            self.publish_to_subscribers();
        }
    }

    pub fn update(&mut self, bid: Option<P>, offer: Option<P>) {
        if *self.price.get_bid() != bid || *self.price.get_offer() != offer {
            self.price = BidOffer::new(bid, offer);
            self.publish_to_subscribers();
        }
    }

    pub fn update_price(&mut self, price: BidOffer<P>) {
        if self.price != price {
            self.price = price;
            self.publish_to_subscribers();
        }
    }

    pub fn update_with_max(
        &mut self,
        bid: Option<P>,
        offer: Option<P>,
        max_bid: Option<A>,
        max_offer: Option<A>,
    ) {
        if *self.price.get_bid() != bid
            || *self.price.get_offer() != offer
            || *self.max.get_bid() != max_bid
            || *self.max.get_offer() != max_offer
        {
            self.price = BidOffer::new(bid, offer);
            self.max = BidOffer::new(max_bid, max_offer);
            self.publish_to_subscribers();
        }
    }

    pub fn update_max(&mut self, max: BidOffer<A>) {
        if self.max != max {
            self.max = max;
            self.publish_to_subscribers();
        }
    }

    pub fn get_price(&self, size: A) -> BidOffer<P> {
        BidOffer::new(
            if self.max.get_bid().map_or(true, |max_size| max_size >= size) {
                *self.price.get_bid()
            } else {
                None
            },
            if self
                .max
                .get_offer()
                .map_or(true, |max_size| max_size >= size)
            {
                *self.price.get_offer()
            } else {
                None
            },
        )
    }

    fn publish_to_subscribers(&self) {
        for &callback in self.callbacks.borrow().iter() {
            callback.market_updated();
        }
    }

    pub fn subscribe(&self, callback: &'a dyn MarketCallback) {
        self.callbacks.borrow_mut().push(callback);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::L1MarketData;
    use crate::market_data::{l1::MarketCallback, BidOffer};

    struct TestCallback {
        called: RefCell<bool>,
    }

    impl TestCallback {
        fn new() -> Self {
            Self {
                called: RefCell::new(false),
            }
        }

        fn reset(&self) {
            *self.called.borrow_mut() = false;
        }
    }

    impl MarketCallback for TestCallback {
        fn market_updated(&self) {
            *self.called.borrow_mut() = true;
        }
    }

    #[test]
    fn max_applied_when_set() {
        let test = L1MarketData::new_with_max(Some(10), Some(10), Some(10), Some(10));

        assert_eq!(test.get_price(9), BidOffer::new(Some(10), Some(10)));
        assert_eq!(test.get_price(10), BidOffer::new(Some(10), Some(10)));
        assert_eq!(test.get_price(11), BidOffer::new(None, None));
    }

    #[test]
    fn max_not_applied_when_not_set() {
        let test = L1MarketData::new_with_max(Some(10), Some(10), Some(10), None);

        assert_eq!(test.get_price(9), BidOffer::new(Some(10), Some(10)));
        assert_eq!(test.get_price(10), BidOffer::new(Some(10), Some(10)));
        assert_eq!(test.get_price(11), BidOffer::new(None, Some(10)));

        let test = L1MarketData::new_with_max(Some(10), Some(10), None, Some(10));

        assert_eq!(test.get_price(9), BidOffer::new(Some(10), Some(10)));
        assert_eq!(test.get_price(10), BidOffer::new(Some(10), Some(10)));
        assert_eq!(test.get_price(11), BidOffer::new(Some(10), None));
    }

    #[test]
    fn change_triggers_subscriptions() {
        let mut test = L1MarketData::new_with_max(Some(10), Some(10), Some(60), Some(70));

        let callback = TestCallback::new();
        test.subscribe(&callback);

        test.update_bid(Some(10));
        assert!(!*callback.called.borrow());

        test.update_bid(Some(9));
        assert!(*callback.called.borrow());

        callback.reset();

        test.update_offer(Some(10));
        assert!(!*callback.called.borrow());

        test.update_offer(Some(19));
        assert!(*callback.called.borrow());

        callback.reset();

        test.update_max_bid(Some(60));
        assert!(!*callback.called.borrow());

        test.update_max_bid(Some(59));
        assert!(*callback.called.borrow());

        callback.reset();

        test.update_max_offer(Some(70));
        assert!(!*callback.called.borrow());

        test.update_max_offer(Some(71));
        assert!(*callback.called.borrow());
    }
}