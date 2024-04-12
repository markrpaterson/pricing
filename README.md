# pricing
A rust based pricing library 

```mermaid
classDiagram
    class BidOffer~P~ {
        Option~P~ bid
        Option~P~ offer
    }

    class MarketLevel~A,P~ {
        A size
        P price
    }

    class Market~A,P~ {
        <<trait>>
        get_price(A size) BidOffer
        get_prices~T~(IntoIterator~A~ sizes) T~A,BidOffer~
    }

    class TieredMarket~A,P~
    Market <|-- TieredMarket

    class SweepMarket~A,P~
    Market <|-- SweepMarket

    class PricingSource~I,A,P~ {
        <<trait>>
        get_market() &Market~A,P~
    }

    class MarketPricingSource~I,M,A,P~ {
        clear()
    }
    PricingSource <|-- MarketPricingSource

    class MarginPricingSource~I,M,A,P~ {
        &PricingSource~I,A,P~ source
    }
    PricingSource <|-- MarginPricingSource
```
