#  cargo test --package simple-fiori-server --lib -- entities::generic::tests::generic_entity_expand_1_1 entities::generic::tests::generic_entity_expand_1n --nocapture
# cargo test --package simple-fiori-server --lib -- entities::generic::tests::generic_entity_expand_unknown_nav_ignored --nocapture 
# cargo test --package simple-fiori-server --lib -- entities::generic::tests --nocapture
cargo test --package simple-fiori-server --test query -- --nocapture
