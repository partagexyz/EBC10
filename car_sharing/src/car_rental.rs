use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct RentalBadge {
    #[mutable]
    start_time: Instant,
    #[mutable]
    duration_in_hours: u32,
}

#[blueprint]
mod car_rental {
    // define auth rules
    enable_method_auth! {
    roles {
        user => updatable_by: [OWNER];
    },
        methods {
            withdraw_car_owner_vault => PUBLIC;
            is_available => restrict_to: [user];
            rent_car => restrict_to: [user];
            return_car => restrict_to: [user];
            collect_fees => restrict_to: [OWNER];
        }
    }
    struct CarRental {
        price_per_hour: Decimal,
        rental_badge_vault: Vault,
        car_owner_vault: Vault,
        car_owner_global_id: NonFungibleGlobalId,
        fees_vault: Vault,
        fee: Decimal,
        // TODO: integrate more information about the car like its location and its mileage
    }

    impl CarRental {
        // Function to instantiate the CarRental blueprint component with initial NFTs
        // TODO: protect it so only the owner of a CarSharing can implement a CarRental
        pub fn instantiate_car_rental(
            component_owner_badge_address: ResourceAddress,
            car_owner_global_id: NonFungibleGlobalId,
            user_badge_address: ResourceAddress,
            price_per_hour: Decimal,
            fee: Decimal,
        ) -> Global<CarRental> {
            // Reserve an address for the component
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(CarRental::blueprint_id());

            // Create a new Rental Badge resource manager with an initial supply of 1
            // The rental badge is a bit like the car key, but with additional information about the rental added to it.
            let rental_badge_bucket: NonFungibleBucket = ResourceBuilder::new_ruid_non_fungible::<
                RentalBadge,
            >(OwnerRole::None)
            .metadata(metadata!(
                init {
                    "name" => "Rental Badge", locked;
                }
            ))
            // Minting is not permitted because we only want at max 1 rental badge per CarRental
            .recall_roles(recall_roles! {
                recaller => rule!(require(component_owner_badge_address,));
                recaller_updater => rule!(deny_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(require(component_owner_badge_address));
                burner_updater => rule!(deny_all);
            })
            .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(deny_all);
            })
            .mint_initial_supply([RentalBadge {
                start_time: Clock::current_time(TimePrecisionV2::Second),
                duration_in_hours: 0,
            }]);

            // Instantiate and return the CarRental component
            Self {
                price_per_hour: price_per_hour,
                rental_badge_vault: Vault::with_bucket(rental_badge_bucket.into()),
                car_owner_global_id: car_owner_global_id,
                car_owner_vault: Vault::new(XRD),
                fees_vault: Vault::new(XRD),
                fee: fee,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                component_owner_badge_address
            ))))
            .roles(roles!(
                user => rule!(require(user_badge_address));
            ))
            .with_address(address_reservation)
            .globalize()
        }

        pub fn withdraw_car_owner_vault(&mut self, car_owner_proof: Proof) -> Bucket {
            // This method is public, but a proof of the right unique CarOwner badge must be given as argument
            // This method checks that the given proof matches with the Car Owner GlobalId associated with the component
            // This check if the Car Owner badges resource addresses are the same
            let checked_proof: CheckedProof =
                car_owner_proof.check(self.car_owner_global_id.resource_address());
            // This checks that the Car Owner badges local ids are the same
            assert_eq!(
                &checked_proof.as_non_fungible().non_fungible_local_id(),
                self.car_owner_global_id.local_id()
            );

            // Return the entirety of the car owner vault
            self.car_owner_vault.take_all()
        }

        pub fn is_available(&self) -> bool {
            !self.rental_badge_vault.is_empty()
        }

        pub fn rent_car(&mut self, duration_in_hours: u32, mut payment_bucket: Bucket) -> Bucket {
            // THis method is restricted to the user role
            // This method rent a car to the caller if its available and if the payment passed is enough to cover the rental price
            // TODO: add a deposit
            assert!(self.is_available(), "The car is not available to rent.");

            // Compute the rental price and check if the user gave enough
            let rental_price: Decimal = self.price_per_hour * duration_in_hours + self.fee;
            assert!(
                payment_bucket.amount() >= rental_price,
                "You did not provided enough XRD for this rental."
            );

            // Take the unique rental badge from the vault
            let rental_badge: Bucket = self.rental_badge_vault.take(1);
            // And update its data with the rental information
            let rental_bag_rm: ResourceManager =
                ResourceManager::from_address(rental_badge.resource_address());
            rental_bag_rm.update_non_fungible_data(
                &rental_badge.as_non_fungible().non_fungible_local_id(),
                "start_time",
                Clock::current_time(TimePrecisionV2::Second),
            );
            rental_bag_rm.update_non_fungible_data(
                &rental_badge.as_non_fungible().non_fungible_local_id(),
                "duration_in_hours",
                duration_in_hours,
            );

            // Take the fee from the payment bucket
            self.fees_vault.put(payment_bucket.take(self.fee));

            // Deposit the rest of the payment bucket to the car owner vault
            self.car_owner_vault.put(payment_bucket);

            // Return the rental badge
            rental_badge
        }

        pub fn return_car(&mut self, rental_badge: Bucket) {
            // THis method is restricted to the user role
            // This method put back the rental badge in the rental badge vault
            // TODO: check if rental duration was respected and if it's overpassed, withdraw from the deposit
            self.rental_badge_vault.put(rental_badge);
        }

        pub fn collect_fees(&mut self) -> Bucket {
            // THis method is restricted to the component owner
            self.fees_vault.take_all()
        }

        // TODO: Add a logic to deactivate the Car Rental (make it unavailable)
    }
}
