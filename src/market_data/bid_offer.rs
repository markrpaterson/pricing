use std::ops::{Add, Div};

/// A structure to hold the pricing for a specific size in the market.  Values are options as there may not be a price for the requested size.
///
/// # Generic Parameters
///
/// * `P` - The Price type that should be used.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BidOffer<P>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
{
    /// Store the bid price
    bid: Option<P>,
    /// Store the bid price
    offer: Option<P>,
}

impl<P> BidOffer<P>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
{
    /// Use the new function to create a new BidOffer which has no pricing
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::BidOffer;
    ///
    /// let bid_offer = BidOffer::<i32>::new();
    /// ```    
    pub fn new() -> Self {
        Self {
            bid: None,
            offer: None,
        }
    }

    /// Use the new function to create a new BidOffer
    ///
    /// # Parameters
    ///
    /// * `bid` - The bid price, a value of None means no price available
    /// * `offer` - The offer price, a value of None means no price available
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::BidOffer;
    ///
    /// let bid_offer = BidOffer::new_with_price(Some(10), Some(20));
    /// ```
    pub fn new_with_price(bid: Option<P>, offer: Option<P>) -> Self {
        Self { bid, offer }
    }

    /// Get the bid price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::BidOffer;
    ///
    /// let bid_offer = BidOffer::new_with_price(Some(10), Some(20));
    ///
    /// assert_eq!(*bid_offer.get_bid(), Some(10));
    /// ```
    pub fn get_bid(&self) -> &Option<P> {
        &self.bid
    }

    /// Get the offer price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::BidOffer;
    ///
    /// let bid_offer = BidOffer::new_with_price(Some(10), Some(20));
    ///
    /// assert_eq!(*bid_offer.get_offer(), Some(20));
    /// ```
    pub fn get_offer(&self) -> &Option<P> {
        &self.offer
    }

    /// Get the mid price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::BidOffer;
    ///
    /// let bid_offer = BidOffer::new_with_price(Some(10), Some(20));
    ///
    /// assert_eq!(bid_offer.get_mid(), Some(15));
    /// ```
    pub fn get_mid(&self) -> Option<P> {
        if let Some(bid) = self.bid {
            if let Some(offer) = self.offer {
                let two: P = 2.into();
                Some((bid + offer) / two)
            } else {
                self.bid
            }
        } else {
            self.offer
        }
    }
}

impl<P> Default for BidOffer<P>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::BidOffer;

    #[test]
    fn basic_double() {
        let bid_offer = BidOffer::new_with_price(Some(1.2), Some(2.4));

        assert_eq!(*bid_offer.get_bid(), Some(1.2));
        assert_eq!(*bid_offer.get_offer(), Some(2.4));
        assert!(((1.2 + 2.4) / 2.0 - bid_offer.get_mid().unwrap()).abs() <= f64::EPSILON);
    }

    #[test]
    fn basic_integer() {
        let bid_offer = BidOffer::new_with_price(Some(12), Some(23));

        assert_eq!(*bid_offer.get_bid(), Some(12));
        assert_eq!(*bid_offer.get_offer(), Some(23));
        assert_eq!(bid_offer.get_mid(), Some((12 + 23) / 2)); // 17 as integer division truncates
    }

    #[test]
    fn mid_test() {
        let bid_offer = BidOffer::new_with_price(Some(12), None);
        assert_eq!(bid_offer.get_mid(), Some(12));

        let bid_offer = BidOffer::new_with_price(None, Some(23));
        assert_eq!(bid_offer.get_mid(), Some(23));

        let bid_offer: BidOffer<i32> = BidOffer::new_with_price(None, None);
        assert_eq!(bid_offer.get_mid(), None);
    }
}
