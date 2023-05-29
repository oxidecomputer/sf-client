// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use std::any::{Any, TypeId};

pub fn is_unit<T: Any>() -> bool {
    TypeId::of::<T>() == TypeId::of::<()>()
}

#[cfg(test)]
mod tests {
    use crate::util::is_unit;

    #[test]
    fn test_types() {
        assert!(is_unit::<()>());
        assert!(!is_unit::<String>());
    }
}