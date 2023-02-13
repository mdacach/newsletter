use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

pub struct NewSubscriber {
    pub(crate) name: SubscriberName,
    pub(crate) email: SubscriberEmail,
}
