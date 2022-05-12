/// A Hexchat plugin preference.
pub trait HexPref {
    /// The name of this preference. This is the identifier passed into Hexchat
    ///     functions to interact with the stored value.
    fn name(&self) -> &str;
}


/// A Hexchat plugin preference with a value that can be read.
pub trait HexPrefGet: HexPref {
    /// This is the type returned by Hexchat. It needs to own its data.
    type Output;

    /// Read the value of this preference.
    fn get(&self) -> Option<Self::Output>;

    /// Read the value of this preference while also unsetting it.
    fn take(&self) -> Option<Result<Self::Output, Self::Output>> where
        Self: HexPrefUnset,
    {
        match self.get() {
            Some(value) => Some(match self.unset() {
                Err(_) => Err(value),
                Ok(_) => Ok(value),
            }),
            None => None,
        }
    }
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
