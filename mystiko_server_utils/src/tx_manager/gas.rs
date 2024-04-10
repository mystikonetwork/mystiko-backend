use ethers_core::types::{I256, U256};
use ethers_core::utils::{
    EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE, EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER,
    EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE,
};

fn estimate_priority_fee(rewards: Vec<Vec<U256>>) -> U256 {
    let mut rewards: Vec<U256> = rewards.iter().map(|r| r[0]).filter(|r| *r > U256::zero()).collect();
    if rewards.is_empty() {
        return U256::zero();
    }
    if rewards.len() == 1 {
        return rewards[0];
    }
    // Sort the rewards as we will eventually take the median.
    rewards.sort();

    // A copy of the same vector is created for convenience to calculate percentage change
    // between subsequent fee values.
    let mut rewards_copy = rewards.clone();
    rewards_copy.rotate_left(1);

    let mut percentage_change: Vec<I256> = rewards
        .iter()
        .zip(rewards_copy.iter())
        .map(|(a, b)| {
            let a = I256::try_from(*a).expect("priority fee overflow");
            let b = I256::try_from(*b).expect("priority fee overflow");
            ((b - a) * 100) / a
        })
        .collect();
    percentage_change.pop();

    // Fetch the max of the percentage change, and that element's index.
    let max_change = percentage_change.iter().max().unwrap();
    let max_change_index = percentage_change.iter().position(|&c| c == *max_change).unwrap();

    // If we encountered a big change in fees at a certain position, then consider only
    // the values >= it.
    let values = if *max_change >= EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE.into()
        && (max_change_index >= (rewards.len() / 2))
    {
        rewards[max_change_index..].to_vec()
    } else {
        rewards
    };

    // Return the median.
    values[values.len() / 2]
}

/// The default EIP-1559 fee estimator which is based on the work by [MyCrypto](https://github.com/MyCryptoHQ/MyCrypto/blob/master/src/services/ApiService/Gas/eip1559.ts)
pub fn eip1559_default_estimator(base_fee_per_gas: U256, rewards: Vec<Vec<U256>>) -> (U256, U256) {
    let max_priority_fee_per_gas = if base_fee_per_gas < U256::from(EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER) {
        U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE)
    } else {
        std::cmp::max(
            estimate_priority_fee(rewards),
            U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE),
        )
    };

    let max_fee_per_gas = if max_priority_fee_per_gas > base_fee_per_gas {
        max_priority_fee_per_gas + base_fee_per_gas
    } else {
        base_fee_per_gas
    };
    (max_fee_per_gas, max_priority_fee_per_gas)
}
