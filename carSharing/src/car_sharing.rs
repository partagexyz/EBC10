use crate::car_rental::car_rental::CarRental;
use scrypto::prelude::*;
#[derive(ScryptoSbor, NonFungibleData)]
pub struct Car {
    // Define what data will be stored for each car NFT
    make: String,
    model: String,
    year: u32,
    vin: String, // unique Vehicle Identification Number
    mileage: u32,
    location: String,
    price_per_hour: Decimal,
    owner_badge_id: NonFungibleLocalId,
}

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
    // define auth rules
    enable_method_auth! {
    roles {
        car_owner => updatable_by: [OWNER];
        user => updatable_by: [OWNER];
    },
        // decide which methods are public and which are restricted to certain roles
        methods {
            create_car_owner_account => PUBLIC;
            create_user_account => PUBLIC;
            add_car => restrict_to: [car_owner];
        }
    }
    struct CarSharing {
        car_owner_badge_resource_manager: ResourceManager,
        user_badge_resource_manager: ResourceManager,
        fee_per_rental: Decimal,
    }

    impl CarSharing {
        // Function to instantiate the CarSharing blueprint component with initial NFTs
        pub fn instantiate_car_sharing() -> (Global<CarSharing>, Bucket) {
            // reserve an address for the component
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(CarSharing::blueprint_id());

            // create an Owner Badge
            let component_owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Car Sharing Component Owner Badge", locked;
                    }
                ))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

            // create a new Car Owner Badge resource manager
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

            let car_owner_badge_dummy_bucket: NonFungibleLocalId = car_owner_badges_manager
                .mint_non_fungible(
                    &NonFungibleLocalId::integer(1),
                    CarOwnerBadge {
                        owner_number: 1,
                        owner_name: "Test".to_string(),
                    },
                )
                .as_non_fungible()
                .non_fungible_local_id();

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

            // Create a new resource called "CarNFT" for non-fungible car tokens
            // TODO: Let's use that to create NFT that represents the ownership of a car
            let cars_bucket: Bucket =
                ResourceBuilder::new_ruid_non_fungible::<Car>(OwnerRole::None)
                    .metadata(metadata! {
                        init {
                            "name" => "CarNFT".to_string(), locked;
                            "symbol" => "CAR".to_string(), locked;
                        }
                    })
                    .mint_roles(mint_roles! {
                        minter => rule!(require(car_owner_badges_manager.address()));
                        minter_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(car_owner_badges_manager.address()));
                        burner_updater => rule!(deny_all);
                    })
                    .mint_initial_supply(
                        // Define initial cars
                        [
                            Car {
                                make: "Tesla".to_string(),
                                model: "Model S".to_string(),
                                year: 2022,
                                vin: "5YJSA1E26MF1XXXXX".to_string(),
                                mileage: 15000,
                                location: "New York".to_string(),
                                price_per_hour: dec!("10"),
                                owner_badge_id: car_owner_badge_dummy_bucket.clone(),
                            },
                            Car {
                                make: "BMW".to_string(),
                                model: "i3".to_string(),
                                year: 2021,
                                vin: "WBY7Z4C57M7XXXXX".to_string(),
                                mileage: 18300,
                                location: "Lisboa".to_string(),
                                price_per_hour: dec!("8"),
                                owner_badge_id: car_owner_badge_dummy_bucket.clone(),
                            },
                            Car {
                                make: "Audi".to_string(),
                                model: "e-tron".to_string(),
                                year: 2023,
                                vin: "WA1LAAGE4LB0XXXXX".to_string(),
                                mileage: 20000,
                                location: "Barcelona".to_string(),
                                price_per_hour: dec!("12"),
                                owner_badge_id: car_owner_badge_dummy_bucket.clone(),
                            },
                            Car {
                                make: "Nissan".to_string(),
                                model: "Leaf".to_string(),
                                year: 2020,
                                vin: "1N4AZ1CP7LC3XXXXX".to_string(),
                                mileage: 23000,
                                location: "Paris".to_string(),
                                price_per_hour: dec!("4"),
                                owner_badge_id: car_owner_badge_dummy_bucket.clone(),
                            },
                            Car {
                                make: "Chevrolet".to_string(),
                                model: "Bolt".to_string(),
                                year: 2021,
                                vin: "1G1FY6S00M4XXXXX".to_string(),
                                mileage: 12000,
                                location: "London".to_string(),
                                price_per_hour: dec!("5"),
                                owner_badge_id: car_owner_badge_dummy_bucket.clone(),
                            },
                        ],
                    )
                    .into();

            // TODO: add resource manager to represent the ownership of a driving license

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let car_sharing_impl: Global<CarSharing> = Self {
                car_owner_badge_resource_manager: car_owner_badges_manager,
                user_badge_resource_manager: user_badges_manager,
                fee_per_rental: dec!("2"),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                component_owner_badge.resource_address()
            ))))
            .roles(roles!(
                car_owner => rule!(require(car_owner_badges_manager.address()));
                user => rule!(require(user_badges_manager.address()));))
            .with_address(address_reservation)
            .globalize();
            (car_sharing_impl, component_owner_badge)
        }

        pub fn create_car_owner_account(
            &mut self,
            name: String,
            number: u64,
            car_proof: String,
        ) -> Bucket {
            assert_eq!(
                car_proof, "Valid Car",
                "You don't have a valid driving license."
            );
            // Mint and receive a new car owner badge.
            let car_owner_badge_bucket: Bucket =
                self.car_owner_badge_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::integer(number),
                    CarOwnerBadge {
                        owner_number: number,
                        owner_name: name,
                    },
                );
            car_owner_badge_bucket
        }
        pub fn create_user_account(
            &mut self,
            name: String,
            number: u64,
            driving_license_proof: String,
        ) -> Bucket {
            assert_eq!(
                driving_license_proof, "Valid Driving License",
                "You don't have a valid driving license."
            );
            // Mint and receive a new user badge.
            let user_badge_bucket: Bucket =
                self.car_owner_badge_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::integer(number),
                    UserBadge {
                        user_number: number,
                        user_name: name,
                        driving_license: driving_license_proof,
                    },
                );
            user_badge_bucket
        }

        // Method to add a new car to the platform
        pub fn add_car(&mut self, car_owner_proof: Proof, car: Car, price_per_hour: Decimal) {
            let checked_proof: CheckedProof =
                car_owner_proof.check(self.car_owner_badge_resource_manager.address());
            let car_owner_badge_global_id = NonFungibleGlobalId::new(
                checked_proof.resource_address(),
                checked_proof.as_non_fungible().non_fungible_local_id(),
            );

            let (car_rental, car_rental_owner_badge) = CarRental::instantiate_car_rental(
                car_owner_badge_global_id,
                self.user_badge_resource_manager.address(),
                price_per_hour,
                self.fee_per_rental,
            );
        }
    }
}
