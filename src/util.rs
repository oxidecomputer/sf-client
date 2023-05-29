// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use serde::de::DeserializeOwned;
use std::any::{Any, TypeId};

use crate::error::{SfResult, Error};

pub fn is_unit<T: Any>() -> bool {
    TypeId::of::<T>() == TypeId::of::<()>()
}

pub fn deser_body<T>(body: &str) -> SfResult<T> where T: DeserializeOwned {
    serde_json::from_str(body).map_err(|error| {
        Error::UnexpectedBody { error, body: body.to_string() }
    })
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