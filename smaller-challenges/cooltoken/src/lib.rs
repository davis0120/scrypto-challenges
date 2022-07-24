use scrypto::prelude::*;

blueprint! {
    struct TokenSale {
        cool_token_vault: Vault,
        xrd_tokens_vault: Vault,
        price_per_token: Decimal,
    }

    impl TokenSale {
        pub fn new(price_per_token: Decimal) -> (ComponentAddress, Bucket) {
            let bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "CoolToken")
                .metadata("symbol", "CT")
                .metadata("team-member-1-ticket-number", "4134631519")
                .metadata("team-member-2-ticket-number", "4053317869")
                .metadata("team-member-3-ticket-number", "4052799999")   
                .metadata("team-member-4-ticket-number", "4052901659")  
                .divisibility(DIVISIBILITY_MAXIMUM)
                .initial_supply(100000);

            let seller_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Seller badge")
                .metadata("symbol", "SELLER")
                .initial_supply(1);

            let access_rules: AccessRules = AccessRules::new()
                .method("withdraw_funds", rule!(require(seller_badge.resource_address())))
                .method("change_address", rule!(require(seller_badge.resource_address())))
                .default(rule!(allow_all));


            let component_address: ComponentAddress = Self {
                cool_token_vault: Vault::with_bucket(bucket),
                xrd_tokens_vault: Vault::new(RADIX_TOKEN),
                price_per_token: price_per_token
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            (component_address, seller_badge)
        }

        pub fn buy(&mut self, funds: Bucket) -> Bucket {
            let purchase_amount: Decimal = funds.amount() / self.price_per_token;
            self.xrd_tokens_vault.put(funds);
            self.cool_token_vault.take(purchase_amount)
        }
        
        pub fn withdraw_funds(&mut self, amount: Decimal) -> Bucket {
            self.xrd_tokens_vault.take(amount) 
        }

        pub fn change_price(&mut self, price: Decimal) {
            self.price_per_token = price
        }
    } 
 }
 