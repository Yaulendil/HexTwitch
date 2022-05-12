use super::pref_trait::*;


pub enum MigrateAction {
    OldValueCleared,
    OldValueMoved,
    NoOldValue,
}


pub enum MigrateFail {
    CannotUnsetOld,
    CannotSetNew,
}


pub struct PrefMigrating<New, Old = New> {
    pub new: New,
    pub old: Old,
}

impl<New, Old> PrefMigrating<New, Old> {
    /// Copy the value of the old preference name to the new preference name,
    ///     and unset the old one.
    pub fn migrate(&self) -> Result<MigrateAction, MigrateFail> where
        New: HexPrefGet + HexPrefSet<<Old as HexPrefGet>::Output>,
        Old: HexPrefGet + HexPrefUnset,
    {
        use MigrateAction::*;
        use MigrateFail::*;

        match self.old.get() {
            //  Old pref is empty. Nothing to do.
            None => Ok(NoOldValue),

            //  Old pref is occupied.
            Some(value) => match self.new.get() {
                Some(_) => {
                    //  New pref already has a value. Unset old.
                    self.old.unset().or(Err(CannotUnsetOld))?;
                    Ok(OldValueCleared)
                }
                None => {
                    //  New pref is empty. Move old into new.
                    self.new.set(value).or(Err(CannotSetNew))?;
                    self.old.unset().or(Err(CannotUnsetOld))?;
                    Ok(OldValueMoved)
                }
            }
        }
    }
}

impl<New, Old> HexPref for PrefMigrating<New, Old> where
    New: HexPref,
{
    fn name(&self) -> &str { self.new.name() }
}

impl<New, Old, Output> HexPrefGet for PrefMigrating<New, Old> where
    New: HexPrefGet<Output=Output>,
    Old: HexPrefGet<Output=Output>,
{
    type Output = Output;

    fn get(&self) -> Option<Self::Output> {
        match self.new.get() {
            Some(val) => Some(val),
            None => self.old.get(),
        }
    }
}

impl<New, Old, Input> HexPrefSet<Input> for PrefMigrating<New, Old> where
    New: HexPrefSet<Input>,
{
    fn set(&self, value: Input) -> Result<(), ()> {
        self.new.set(value)
    }
}

impl<New, Old> HexPrefUnset for PrefMigrating<New, Old> where
    New: HexPrefUnset,
    Old: HexPrefUnset,
{
    fn unset(&self) -> Result<(), ()> {
        let unset_new = self.new.unset();
        let unset_old = self.old.unset();

        unset_new.and(unset_old)
    }
}
