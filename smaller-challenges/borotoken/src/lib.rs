use scrypto::prelude::*;

blueprint! {
   struct TokenSale {
        boro_tokens_vault: Vault,
        xrd_tokens_vault: Vault,
        price_per_token: Decimal,
   }

   impl TokenSale {
       pub fn new(price_per_token: Decimal) -> (ComponentAddress, Bucket) {
        let bucket: Bucket = ResourceBuilder::new_fungible()
            .metadata("name", "Boro Token")
            .metadata("team-member-1-ticket-number", "#4054353449")
            .initial_supply(100000);

        let seller_badge: Bucket = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_MAXIMUM)
            .metadata("name", "Seller Badge")
            .metadata("symbol", "SELLER")
            .initial_supply(1);

        let access_rules: AccessRules = AccessRules::new()
            .method("withdraw_funds", rule!(require(seller_badge.resource_address())))
            .method("change_price", rule!(require(seller_badge.resource_address())))
            .default(rule!(allow_all));

        let component_address: ComponentAddress = Self {
            boro_tokens_vault: Vault::with_bucket(bucket),
            xrd_tokens_vault: Vault::new(RADIX_TOKEN),
            price_per_token: price_per_token,
        }
        .instantiate()
        .add_access_check(access_rules)
        .globalize();

        (component_address, seller_badge)
       }

       pub fn buy(&mut self, funds: Bucket) -> Bucket {
           let purchase_amount: Decimal = funds.amount() / self.price_per_token;
           self.xrd_tokens_vault.put(funds);
           self.boro_tokens_vault.take(purchase_amount)
       }

       pub fn withdraw_funds(&mut self, amount: Decimal) -> Bucket {
           self.xrd_tokens_vault.take(amount)
       }

       pub fn change_price(&mut self, price: Decimal) {
            self.price_per_token = price
       }
   }
}
