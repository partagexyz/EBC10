use crate::car_rental::car_rental::CarRental;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct CarOwnerBadge {
    owner_number: u64,
    owner_name: String,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct UserBadge {
    user_number: u64,
    user_name: String,
    driving_license: String,
}

#[blueprint]
mod car_sharing {
    struct CarSharing {
        component_owner_badge_address: ResourceAddress,
        car_owner_badge_resource_manager: ResourceManager,
        user_badge_resource_manager: ResourceManager,
        fee_per_rental: Decimal,
    }

    impl CarSharing {
        // Function to instantiate the CarSharing blueprint component with
        pub fn instantiate_car_sharing() -> (Global<CarSharing>, Bucket) {
            // Reserve an address for the component
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(CarSharing::blueprint_id());

            // Create a Component Owner Badge
            let component_owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "CarSharing Component Owner Badge", locked;
                    }
                ))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

            // Create a new Car Owner Badge resource manager
            let car_owner_badges_manager: ResourceManager =
                ResourceBuilder::new_integer_non_fungible::<CarOwnerBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Car Owner Badge", locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(component_owner_badge.resource_address()));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(component_owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            // create a new User Badge resource manager
            let user_badges_manager: ResourceManager =
                ResourceBuilder::new_integer_non_fungible::<UserBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "User Badge", locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(component_owner_badge.resource_address(),));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(component_owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            // TODO: add NF resource manager to represent the ownership of a car (this should be part of another component)
            // TODO: add NF resource manager to represent the ownership of a valid driving license (this should be part of another component)

            // Instantiate a CarSharing component
            let car_sharing_impl: Global<CarSharing> = Self {
                component_owner_badge_address: component_owner_badge.resource_address(),
                car_owner_badge_resource_manager: car_owner_badges_manager,
                user_badge_resource_manager: user_badges_manager,
                fee_per_rental: dec!("2"),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                component_owner_badge.resource_address()
            ))))
            .with_address(address_reservation)
            .globalize();

            // Return the instantiated component and its owner badge
            (car_sharing_impl, component_owner_badge)
        }

        pub fn create_car_owner_account(
            &mut self,
            name: String,
            number: u64,
            car_proof: String,
        ) -> Bucket {
            // This method is public
            // This method mints a Car Owner badge containing the information passed as argument
            // TODO: change for a real car ownership NFT (managed by another component)
            assert_eq!(
                car_proof, "Valid Car",
                "You don't have a valid car ownership proof."
            );
            // Mint a new car owner badge.
            let car_owner_badge_bucket: Bucket =
                self.car_owner_badge_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::integer(number),
                    CarOwnerBadge {
                        owner_number: number,
                        owner_name: name,
                    },
                );
            // Return the car owner badge
            car_owner_badge_bucket
        }
        pub fn create_user_account(
            &mut self,
            name: String,
            number: u64,
            driving_license_proof: String,
        ) -> Bucket {
            // This method is public
            // This method mints a User badge containing the information passed as argument
            // TODO: change for a real driving license ownership NFT (managed by another component)
            assert_eq!(
                driving_license_proof, "Valid Driving License",
                "You don't have a valid driving license."
            );
            // Mint a new user badge.
            let user_badge_bucket: Bucket = self.user_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(number),
                UserBadge {
                    user_number: number,
                    user_name: name,
                    driving_license: driving_license_proof,
                },
            );
            // Return the new user badge
            user_badge_bucket
        }

        // Method to add a new car to the platform
        pub fn add_car(
            &mut self,
            car_owner_proof: Proof,
            price_per_hour: Decimal,
            car_proof: String,
            // TODO: Add information about the car
        ) {
            // This method is public but requires a Car Owner proof as argument
            // This function checks that a proof of Car Owner badge is given before instantiating a CarRental component
            let checked_proof: CheckedProof =
                car_owner_proof.check(self.car_owner_badge_resource_manager.address());

            // TODO: change for a real car ownership NFT (managed by another component)
            assert_eq!(
                car_proof, "Valid Car",
                "You don't have a valid proof of car ownership."
            );

            // Retrieve the Car Owner badge global id to use it as a authentication proof in the CarRental
            let car_owner_badge_global_id: NonFungibleGlobalId = NonFungibleGlobalId::new(
                checked_proof.resource_address(),
                checked_proof.as_non_fungible().non_fungible_local_id(),
            );

            CarRental::instantiate_car_rental(
                self.component_owner_badge_address,
                car_owner_badge_global_id,
                self.user_badge_resource_manager.address(),
                price_per_hour,
                self.fee_per_rental,
            );
        }

        // TODO: In another back end component (off chain), implement a function for the user to list all the available car to rent
    }
}
