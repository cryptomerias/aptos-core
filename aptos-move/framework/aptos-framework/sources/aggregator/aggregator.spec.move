spec aptos_framework::aggregator {
    spec Aggregator {
        pragma intrinsic;
    }

    // TODO: Write spec for native_add and native_sub. 
    spec try_add(aggregator: &mut Aggregator, value: u128): bool {
        pragma opaque;
        ensures spec_aggregator_get_val(aggregator) + value > spec_get_limit(aggregator) ==> result == false;
        ensures spec_aggregator_get_val(aggregator) + value > MAX_U128 ==> result == false;
        ensures spec_get_limit(aggregator) == spec_get_limit(old(aggregator));
        ensures result == true ==> aggregator == spec_aggregator_set_val(old(aggregator),
            spec_aggregator_get_val(old(aggregator)) + value);
    }

    spec try_sub(aggregator: &mut Aggregator, value: u128): bool {
        pragma opaque;
        ensures spec_aggregator_get_val(aggregator) < value ==> result == false;
        ensures spec_get_limit(aggregator) == spec_get_limit(old(aggregator));
        ensures result == true ==> aggregator == spec_aggregator_set_val(old(aggregator),
            spec_aggregator_get_val(old(aggregator)) - value);
    }

    spec add(aggregator: &mut Aggregator, value: u128) {
        pragma opaque;
        aborts_if spec_aggregator_get_val(aggregator) + value > spec_get_limit(aggregator);
        aborts_if spec_aggregator_get_val(aggregator) + value > MAX_U128;
        ensures spec_get_limit(aggregator) == spec_get_limit(old(aggregator));
        ensures aggregator == spec_aggregator_set_val(old(aggregator),
            spec_aggregator_get_val(old(aggregator)) + value);
    }

    spec sub(aggregator: &mut Aggregator, value: u128) {
        pragma opaque;
        aborts_if spec_aggregator_get_val(aggregator) < value;
        ensures spec_get_limit(aggregator) == spec_get_limit(old(aggregator));
        ensures aggregator == spec_aggregator_set_val(old(aggregator),
            spec_aggregator_get_val(old(aggregator)) - value);
    }

    spec read(aggregator: &Aggregator): u128 {
        pragma opaque;
        aborts_if false;
        ensures result == spec_read(aggregator);
        ensures result <= spec_get_limit(aggregator);
    }

    spec destroy(aggregator: Aggregator) {
        pragma opaque;
        aborts_if false;
    }

    spec native fun spec_read(aggregator: Aggregator): u128;
    spec native fun spec_get_limit(a: Aggregator): u128;
    spec native fun spec_get_handle(a: Aggregator): u128;
    spec native fun spec_get_key(a: Aggregator): u128;
    spec native fun spec_aggregator_set_val(a: Aggregator, v: u128): Aggregator;
    spec native fun spec_aggregator_get_val(a: Aggregator): u128;
}
