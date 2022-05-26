/// A `Result` where both `Ok` and `Err` contain the same `Option` type. Meant
///     to represent a case where an indicator of success or failure is needed,
///     but the same value is returned in either case.
#[allow(dead_code)] // Possibly bugged lint?
type ResOpt<T> = Result<Option<T>, Option<T>>;


/// A Hexchat plugin preference.
pub trait HexPref {
    /// The name of this preference. This is the identifier passed into Hexchat
    ///     functions to interact with the stored value.
    fn name(&self) -> &str;

    /// Check whether a value is equal to the current value of this preference.
    ///     Returns `false` if this preference is not set.
    fn is<T>(&self, value: &T) -> bool where
        Self: HexPrefGet,
        <Self as HexPrefGet>::Output: PartialEq<T>,
    {
        match self.get() {
            Some(current) => current.eq(value),
            None => false,
        }
    }

    /// Set a new value for this preference, and return the previous value.
    fn replace<T>(&self, new: T) -> ResOpt<Self::Output> where
        Self: HexPrefGet + HexPrefSet<T>,
    {
        let old = self.get();

        match self.set(new) {
            Err(_) => Err(old),
            Ok(_) => Ok(old),
        }
    }

    /// Read the value of this preference while also unsetting it.
    fn take(&self) -> ResOpt<Self::Output> where
        Self: HexPrefGet + HexPrefUnset,
    {
        match self.get() {
            Some(value) => match self.unset() {
                Err(_) => Err(Some(value)),
                Ok(_) => Ok(Some(value)),
            }
            None => Ok(None),
        }
    }
}


/// A Hexchat plugin preference with a value that can be read.
pub trait HexPrefGet: HexPref {
    /// This is the type returned by Hexchat. It needs to own its data.
    type Output;

    /// Read the value of this preference.
    fn get(&self) -> Option<Self::Output>;
}


/// A Hexchat plugin preference whose value can be written.
pub trait HexPrefSet<Input>: HexPref {
    /// Write a new value to this preference, saving it in the configuration
    ///     data of Hexchat itself.
    fn set(&self, value: Input) -> Result<(), ()>;
}


/// A Hexchat plugin preference whose value can be removed.
pub trait HexPrefUnset: HexPref {
    /// Unset this preference, leaving it with no value.
    fn unset(&self) -> Result<(), ()> {
        hexchat::delete_pref(self.name())
    }
}
