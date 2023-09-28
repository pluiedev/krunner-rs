Registers the KRunner interface for this runner to a [`Crossroads`](dbus_crossroads::Crossroads)
instance.

Usually, you can just use the [`start`](Self::start) method to start the
runner. However, if you ever need to customize the D-Bus connection or how
exactly the D-Bus server should react to events, this is the method to
call.
