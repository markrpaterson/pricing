use std::{
    cell::RefCell,
    ops::{Add, Div},
    rc::Rc,
};

use super::BidOffer;

pub trait L1MarketCallback {
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
pub struct L1MarketData<P, A>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
    A: Copy + PartialOrd + Add<Output = A> + Div<Output = A> + From<i32>,
{
    price: BidOffer<P>,
    max: BidOffer<A>,

    callbacks: RefCell<Vec<Rc<dyn L1MarketCallback>>>,
}

impl<P, A> L1MarketData<P, A>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
    A: Copy + PartialOrd + Add<Output = A> + Div<Output = A> + From<i32>,
{
    /// Use the new function to create a new L1MarketData with no pricing.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let market_data = L1MarketData::<i32, i32>::new();
    ///
    /// assert_eq!(*market_data.get_bid(), None);
    /// assert_eq!(*market_data.get_offer(), None);
    /// assert_eq!(*market_data.get_max_bid(), None);
    /// assert_eq!(*market_data.get_max_offer(), None);
    /// ```
    pub fn new() -> Self {
        Self {
            price: BidOffer::new(),
            max: BidOffer::new(),
            callbacks: RefCell::new(Vec::new()),
        }
    }

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
    /// let market_data = L1MarketData::<_, i32>::new_with_price(Some(10), Some(20));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    /// assert_eq!(*market_data.get_offer(), Some(20));
    /// assert_eq!(*market_data.get_max_bid(), None);
    /// assert_eq!(*market_data.get_max_offer(), None);
    /// ```
    pub fn new_with_price(bid: Option<P>, offer: Option<P>) -> Self {
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
    /// let market_data = L1MarketData::new_with_max(Some(10), Some(20), Some(100), Some(101));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    /// assert_eq!(*market_data.get_offer(), Some(20));
    /// assert_eq!(*market_data.get_max_bid(), Some(100));
    /// assert_eq!(*market_data.get_max_offer(), Some(101));
    /// ```
    pub fn new_with_max(
        bid: Option<P>,
        offer: Option<P>,
        max_bid: Option<A>,
        max_offer: Option<A>,
    ) -> Self {
        Self {
            price: BidOffer::new_with_price(bid, offer),
            max: BidOffer::new_with_price(max_bid, max_offer),
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
    /// let market_data = L1MarketData::<_, i32>::new_with_price(Some(10), Some(20));
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
    /// let market_data = L1MarketData::<_, i32>::new_with_price(Some(10), Some(20));
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
    /// let market_data = L1MarketData::<_, i32>::new_with_price(Some(10), Some(20));
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
    /// # Parameters
    ///
    /// * `bid` - The bid price, a value of None means no price available
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let mut market_data = L1MarketData::<_, i32>::new_with_price(Some(10), Some(20));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    ///
    /// market_data.update_bid(Some(12));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(12));
    /// ```
    pub fn update_bid(&mut self, bid: Option<P>) {
        if *self.price.get_bid() != bid {
            self.price = BidOffer::new_with_price(bid, *self.price.get_offer());
            self.publish_to_subscribers();
        }
    }

    /// Update the offer price
    ///
    /// # Parameters
    ///
    /// * `offer` - The offer price, a value of None means no price available
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let mut market_data = L1MarketData::<_, i32>::new_with_price(Some(10), Some(20));
    ///
    /// assert_eq!(*market_data.get_offer(), Some(20));
    ///
    /// market_data.update_offer(Some(22));
    ///
    /// assert_eq!(*market_data.get_offer(), Some(22));
    /// ```
    pub fn update_offer(&mut self, offer: Option<P>) {
        if *self.price.get_offer() != offer {
            self.price = BidOffer::new_with_price(*self.price.get_bid(), offer);
            self.publish_to_subscribers();
        }
    }

    /// Update the maximum size the bid is valid for
    ///
    /// # Parameters
    ///
    /// * `max_bid` - The max size that the bid price is valid.  A value of None means there is no limit.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let mut market_data = L1MarketData::<_, i32>::new_with_max(Some(10), Some(20), Some(40), Some(50));
    ///
    /// assert_eq!(*market_data.get_max_bid(), Some(40));
    ///
    /// market_data.update_max_bid(Some(42));
    ///
    /// assert_eq!(*market_data.get_max_bid(), Some(42));
    /// ```
    pub fn update_max_bid(&mut self, max_bid: Option<A>) {
        if *self.max.get_bid() != max_bid {
            self.max = BidOffer::new_with_price(max_bid, *self.max.get_offer());
            self.publish_to_subscribers();
        }
    }

    /// Update the maximum size the offer is valid for
    ///
    /// # Parameters
    ///
    /// * `max_offer` - The max size that the offer price is valid.  A value of None means there is no limit.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let mut market_data = L1MarketData::<_, i32>::new_with_max(Some(10), Some(20), Some(40), Some(50));
    ///
    /// assert_eq!(*market_data.get_max_offer(), Some(50));
    ///
    /// market_data.update_max_offer(Some(52));
    ///
    /// assert_eq!(*market_data.get_max_offer(), Some(52));
    /// ```
    pub fn update_max_offer(&mut self, max_offer: Option<A>) {
        if *self.max.get_offer() != max_offer {
            self.max = BidOffer::new_with_price(*self.max.get_bid(), max_offer);
            self.publish_to_subscribers();
        }
    }

    /// Update both the bid and offer prices
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
    /// let mut market_data = L1MarketData::<_, i32>::new_with_price(Some(10), Some(20));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    /// assert_eq!(*market_data.get_offer(), Some(20));
    ///
    /// market_data.update(Some(12), Some(22));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(12));
    /// assert_eq!(*market_data.get_offer(), Some(22));
    /// ```
    pub fn update(&mut self, bid: Option<P>, offer: Option<P>) {
        if *self.price.get_bid() != bid || *self.price.get_offer() != offer {
            self.price = BidOffer::new_with_price(bid, offer);
            self.publish_to_subscribers();
        }
    }

    /// Update both the bid and offer prices using a bid/offer structure
    ///
    /// # Parameters
    ///
    /// * `price` - The bid/offer to update the price with
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::{BidOffer, L1MarketData};
    ///
    /// let mut market_data = L1MarketData::<_, i32>::new_with_price(Some(10), Some(20));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    /// assert_eq!(*market_data.get_offer(), Some(20));
    ///
    /// market_data.update_price(BidOffer::new_with_price(Some(12), Some(22)));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(12));
    /// assert_eq!(*market_data.get_offer(), Some(22));
    /// ```
    pub fn update_price(&mut self, price: BidOffer<P>) {
        if self.price != price {
            self.price = price;
            self.publish_to_subscribers();
        }
    }

    /// Update the price and maximum sizes in a single call
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
    /// let mut market_data = L1MarketData::new_with_max(Some(10), Some(20), Some(40), Some(50));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    /// assert_eq!(*market_data.get_offer(), Some(20));
    /// assert_eq!(*market_data.get_max_bid(), Some(40));
    /// assert_eq!(*market_data.get_max_offer(), Some(50));
    ///
    /// market_data.update_with_max(Some(12), Some(22), Some(42), Some(52));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(12));
    /// assert_eq!(*market_data.get_offer(), Some(22));
    /// assert_eq!(*market_data.get_max_bid(), Some(42));
    /// assert_eq!(*market_data.get_max_offer(), Some(52));
    /// ```
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
            self.price = BidOffer::new_with_price(bid, offer);
            self.max = BidOffer::new_with_price(max_bid, max_offer);
            self.publish_to_subscribers();
        }
    }

    /// Update both the bid and offer max size using a bid/offer structure
    ///
    /// # Parameters
    ///
    /// * `max` - The max size as a bid/offer structure.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::{BidOffer, L1MarketData};
    ///
    /// let mut market_data = L1MarketData::new_with_max(Some(10), Some(20), Some(40), Some(50));
    ///
    /// assert_eq!(*market_data.get_max_bid(), Some(40));
    /// assert_eq!(*market_data.get_max_offer(), Some(50));
    ///
    /// market_data.update_max(BidOffer::new_with_price(Some(42), Some(52)));
    ///
    /// assert_eq!(*market_data.get_max_bid(), Some(42));
    /// assert_eq!(*market_data.get_max_offer(), Some(52));
    /// ```
    pub fn update_max(&mut self, max: BidOffer<A>) {
        if self.max != max {
            self.max = max;
            self.publish_to_subscribers();
        }
    }

    /// Update the both the price and the max sizes using bid/offer structures
    ///
    /// # Parameters
    ///
    /// * `price` - The bid/offer to update the price with.
    /// * `max` - The max size as a bid/offer structure.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::{BidOffer, L1MarketData};
    ///
    /// let mut market_data = L1MarketData::<_, i32>::new_with_max(Some(10), Some(20), Some(40), Some(50));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    /// assert_eq!(*market_data.get_offer(), Some(20));
    /// assert_eq!(*market_data.get_max_bid(), Some(40));
    /// assert_eq!(*market_data.get_max_offer(), Some(50));
    ///
    /// market_data.update_price_with_max(BidOffer::new_with_price(Some(12),Some(22)),BidOffer::new_with_price(Some(42),Some(52)));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(12));
    /// assert_eq!(*market_data.get_offer(), Some(22));
    /// assert_eq!(*market_data.get_max_bid(), Some(42));
    /// assert_eq!(*market_data.get_max_offer(), Some(52));
    /// ```
    pub fn update_price_with_max(&mut self, price: BidOffer<P>, max: BidOffer<A>) {
        if self.price != price || self.max != max {
            self.price = price;
            self.max = max;
            self.publish_to_subscribers();
        }
    }

    /// Clears the data structure setting bid/off and the max sizes all to None
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L1MarketData;
    ///
    /// let mut market_data = L1MarketData::<_, i32>::new_with_max(Some(10), Some(20), Some(40), Some(50));
    ///
    /// assert_eq!(*market_data.get_bid(), Some(10));
    /// assert_eq!(*market_data.get_offer(), Some(20));
    /// assert_eq!(*market_data.get_max_bid(), Some(40));
    /// assert_eq!(*market_data.get_max_offer(), Some(50));
    ///
    /// market_data.clear();
    ///
    /// assert_eq!(*market_data.get_bid(), None);
    /// assert_eq!(*market_data.get_offer(), None);
    /// assert_eq!(*market_data.get_max_bid(), None);
    /// assert_eq!(*market_data.get_max_offer(), None);
    /// ```
    pub fn clear(&mut self) {
        if self.price.get_bid().is_some()
            || self.price.get_offer().is_some()
            || self.max.get_bid().is_some()
            || self.max.get_offer().is_some()
        {
            self.price = BidOffer::new();
            self.max = BidOffer::new();
            self.publish_to_subscribers();
        }
    }

    /// Returns the price for the size passed in.  For an L1 market this is pretty straight forward, as the bid/offer
    /// returned is the current bid/offer unless the size exceeds the max size.
    ///
    /// # Parameters
    ///
    /// * `size` - The size the price is required for
    ///
    /// # Returns
    ///
    /// A Bid/Offer structure with the price
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::{BidOffer, L1MarketData};
    ///
    /// let mut market_data = L1MarketData::<_, i32>::new_with_max(Some(10), Some(20), Some(40), Some(50));
    ///
    /// assert_eq!(market_data.get_price(40), BidOffer::new_with_price(Some(10), Some(20)));
    /// assert_eq!(market_data.get_price(50), BidOffer::new_with_price(None, Some(20)));
    /// assert_eq!(market_data.get_price(55), BidOffer::new_with_price(None, None));
    /// ```
    pub fn get_price(&self, size: A) -> BidOffer<P> {
        BidOffer::new_with_price(
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

    /// Subscribe to changes to the pricing, and is only called if the pricing actually changes (i.e. updating with the current
    /// value will not trigger the subscription)  NOTE: this will occur in the same thread as the caller, so make sure that this
    /// does not cause a recursion issue.
    ///
    /// # Parameters
    ///
    /// * `callback` - The object which implements the L1MarketCallback trait to callback on
    ///
    /// # Example
    ///
    /// ```
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    /// use pricing::market_data::{L1MarketCallback, L1MarketData};
    ///
    /// struct TestCallback {
    ///     called: RefCell<bool>,
    /// }
    ///
    /// impl TestCallback {
    ///     fn new() -> Self {
    ///         Self {
    ///             called: RefCell::new(false),
    ///         }
    ///     }
    ///
    ///     fn reset(&self) {
    ///         *self.called.borrow_mut() = false;
    ///     }
    ///
    ///     fn is_called(&self) -> bool {
    ///         *self.called.borrow()
    ///     }
    /// }
    ///
    /// impl L1MarketCallback for TestCallback {
    ///     fn market_updated(&self) {
    ///         *self.called.borrow_mut() = true;
    ///     }
    /// }
    ///
    /// let mut test = L1MarketData::<_, i32>::new_with_price(Some(10), Some(10));
    /// let callback = Rc::new(TestCallback::new());
    /// test.subscribe(callback.clone());
    ///
    /// test.update_bid(Some(10));
    /// assert!(!callback.is_called());
    /// test.update_bid(Some(9));
    /// assert!(callback.is_called());
    /// ```
    pub fn subscribe(&self, callback: Rc<dyn L1MarketCallback>) {
        self.callbacks.borrow_mut().push(callback.clone());
    }

    fn publish_to_subscribers(&self) {
        for callback in self.callbacks.borrow().iter() {
            callback.market_updated();
        }
    }
}

impl<P, A> Default for L1MarketData<P, A>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
    A: Copy + PartialOrd + Add<Output = A> + Div<Output = A> + From<i32>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;
    use crate::market_data::BidOffer;

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

        fn is_called(&self) -> bool {
            *self.called.borrow()
        }
    }

    impl L1MarketCallback for TestCallback {
        fn market_updated(&self) {
            *self.called.borrow_mut() = true;
        }
    }

    #[test]
    fn max_applied_when_set() {
        let test = L1MarketData::new_with_max(Some(10), Some(10), Some(10), Some(10));

        assert_eq!(
            test.get_price(9),
            BidOffer::new_with_price(Some(10), Some(10))
        );
        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(10), Some(10))
        );
        assert_eq!(test.get_price(11), BidOffer::new());
    }

    #[test]
    fn max_not_applied_when_not_set() {
        let test = L1MarketData::new_with_max(Some(10), Some(10), Some(10), None);

        assert_eq!(
            test.get_price(9),
            BidOffer::new_with_price(Some(10), Some(10))
        );
        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(10), Some(10))
        );
        assert_eq!(test.get_price(11), BidOffer::new_with_price(None, Some(10)));

        let test = L1MarketData::new_with_max(Some(10), Some(10), None, Some(10));

        assert_eq!(
            test.get_price(9),
            BidOffer::new_with_price(Some(10), Some(10))
        );
        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(10), Some(10))
        );
        assert_eq!(test.get_price(11), BidOffer::new_with_price(Some(10), None));
    }

    #[test]
    fn update_individual_triggers_subscriptions() {
        let mut test = L1MarketData::new_with_max(Some(10), Some(10), Some(60), Some(70));

        let callback = Rc::new(TestCallback::new());
        test.subscribe(callback.clone());

        test.update_bid(Some(10));
        assert!(!callback.is_called());
        test.update_bid(Some(9));
        assert!(callback.is_called());

        callback.reset();
        test.update_offer(Some(10));
        assert!(!callback.is_called());
        test.update_offer(Some(19));
        assert!(callback.is_called());

        callback.reset();
        test.update_max_bid(Some(60));
        assert!(!callback.is_called());
        test.update_max_bid(Some(59));
        assert!(callback.is_called());

        callback.reset();
        test.update_max_offer(Some(70));
        assert!(!callback.is_called());
        test.update_max_offer(Some(71));
        assert!(callback.is_called());
    }

    #[test]
    fn update_multiple_triggers_subscriptions() {
        let mut test = L1MarketData::new_with_max(Some(10), Some(10), Some(60), Some(70));

        let callback = Rc::new(TestCallback::new());
        test.subscribe(callback.clone());

        test.update(Some(10), Some(10));
        assert!(!callback.is_called());
        test.update(Some(12), Some(10));
        assert!(callback.is_called());
        callback.reset();
        test.update(Some(12), Some(12));
        assert!(callback.is_called());

        callback.reset();
        test.update_price(BidOffer::new_with_price(Some(12), Some(12)));
        assert!(!callback.is_called());
        test.update_price(BidOffer::new_with_price(Some(11), Some(12)));
        assert!(callback.is_called());
        callback.reset();
        test.update_price(BidOffer::new_with_price(Some(11), Some(11)));
        assert!(callback.is_called());

        callback.reset();
        test.update_max(BidOffer::new_with_price(Some(60), Some(70)));
        assert!(!callback.is_called());
        test.update_max(BidOffer::new_with_price(Some(61), Some(70)));
        assert!(callback.is_called());
        callback.reset();
        test.update_max(BidOffer::new_with_price(Some(61), Some(71)));
        assert!(callback.is_called());

        callback.reset();
        test.update_price_with_max(
            BidOffer::new_with_price(Some(11), Some(11)),
            BidOffer::new_with_price(Some(61), Some(71)),
        );
        assert!(!callback.is_called());
        test.update_price_with_max(
            BidOffer::new_with_price(Some(12), Some(11)),
            BidOffer::new_with_price(Some(61), Some(71)),
        );
        assert!(callback.is_called());
        callback.reset();
        test.update_price_with_max(
            BidOffer::new_with_price(Some(12), Some(12)),
            BidOffer::new_with_price(Some(61), Some(71)),
        );
        assert!(callback.is_called());
        callback.reset();
        test.update_price_with_max(
            BidOffer::new_with_price(Some(12), Some(12)),
            BidOffer::new_with_price(Some(62), Some(71)),
        );
        assert!(callback.is_called());
        callback.reset();
        test.update_price_with_max(
            BidOffer::new_with_price(Some(12), Some(12)),
            BidOffer::new_with_price(Some(62), Some(72)),
        );
        assert!(callback.is_called());
    }

    #[test]
    fn clear_triggers_subscriptions() {
        let mut test = L1MarketData::new();

        let callback = Rc::new(TestCallback::new());
        test.subscribe(callback.clone());

        test.clear();
        assert!(!callback.is_called());

        test.update_with_max(Some(10), Some(10), Some(60), Some(70));
        callback.reset();

        test.clear();
        assert!(callback.is_called());

        test.update_with_max(Some(10), None, None, None);
        callback.reset();
        test.clear();
        assert!(callback.is_called());

        test.update_with_max(None, Some(10), None, None);
        callback.reset();
        test.clear();
        assert!(callback.is_called());

        test.update_with_max(None, None, Some(10), None);
        callback.reset();
        test.clear();
        assert!(callback.is_called());

        test.update_with_max(None, None, None, Some(10));
        callback.reset();
        test.clear();
        assert!(callback.is_called());
    }
}
