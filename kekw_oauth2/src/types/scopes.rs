//! Adapted from:
//! <https://github.com/twitch-rs/twitch_oauth2/blob/e8bfe4e80e4c5a53f1b0ed77cf85db0fcde3aa31/src/scopes.rs>

use std::str::FromStr;

use kekw_macros::{
    DebugExprs, DerefNewType, DeserializeFromStr, DisplayStrings, NewTypeFrom, VariantFromStr,
    VariantStrings,
};

#[derive(
    Copy, Clone, DebugExprs, DisplayStrings, VariantStrings, VariantFromStr, DeserializeFromStr,
)]
pub enum Scope {
    /// View analytics data for the Twitch Extensions owned by the authenticated account.
    #[static_str("analytics:read:extensions")]
    AnalyticsReadExtensions,
    /// View analytics data for the games owned by the authenticated account.
    #[static_str("analytics:read:games")]
    AnalyticsReadGames,
    /// View Bits information for a channel.
    #[static_str("bits:read")]
    BitsRead,
    /// Allows the client’s bot users access to a channel.
    #[static_str("channel:bot")]
    ChannelBot,
    /// Run commercials on a channel.
    #[static_str("channel:edit:commercial")]
    ChannelEditCommercial,
    /// Manage ads schedule on a channel.
    #[static_str("channel:manage:ads")]
    ChannelManageAds,
    /// Manage a channel’s broadcast configuration, including updating channel configuration and managing stream markers and stream tags.
    #[static_str("channel:manage:broadcast")]
    ChannelManageBroadcast,
    /// Manage a channel’s Extension configuration, including activating Extensions.
    #[static_str("channel:manage:extensions")]
    ChannelManageExtensions,
    /// Add or remove the moderator role from users in your channel.
    #[static_str("channel:manage:moderators")]
    ChannelManageModerators,
    /// Manage a channel’s polls.
    #[static_str("channel:manage:polls")]
    ChannelManagePolls,
    /// Manage of channel’s Channel Points Predictions
    #[static_str("channel:manage:predictions")]
    ChannelManagePredictions,
    /// Manage a channel raiding another channel.
    #[static_str("channel:manage:raids")]
    ChannelManageRaids,
    /// Manage Channel Points custom rewards and their redemptions on a channel.
    #[static_str("channel:manage:redemptions")]
    ChannelManageRedemptions,
    /// Manage a channel’s stream schedule.
    #[static_str("channel:manage:schedule")]
    ChannelManageSchedule,
    /// Manage a channel’s videos, including deleting videos.
    #[static_str("channel:manage:videos")]
    ChannelManageVideos,
    /// Add or remove the VIP role from users in your channel.
    #[static_str("channel:manage:vips")]
    ChannelManageVips,
    /// Perform moderation actions in a channel. The user requesting the scope must be a moderator in the channel.
    #[static_str("channel:moderate")]
    ChannelModerate,
    /// Read the ads schedule and details on your channel.
    #[static_str("channel:read:ads")]
    ChannelReadAds,
    /// Read charity campaign details and user donations on your channel.
    #[static_str("channel:read:charity")]
    ChannelReadCharity,
    /// View a list of users with the editor role for a channel.
    #[static_str("channel:read:editors")]
    ChannelReadEditors,
    /// View Creator Goals for a channel.
    #[static_str("channel:read:goals")]
    ChannelReadGoals,
    /// View Hype Train information for a channel.
    #[static_str("channel:read:hype_train")]
    ChannelReadHypeTrain,
    /// View a channel’s polls.
    #[static_str("channel:read:polls")]
    ChannelReadPolls,
    /// View a channel’s Channel Points Predictions.
    #[static_str("channel:read:predictions")]
    ChannelReadPredictions,
    /// View Channel Points custom rewards and their redemptions on a channel.
    #[static_str("channel:read:redemptions")]
    ChannelReadRedemptions,
    /// View an authorized user’s stream key.
    #[static_str("channel:read:stream_key")]
    ChannelReadStreamKey,
    /// View a list of all subscribers to a channel and check if a user is subscribed to a channel.
    #[static_str("channel:read:subscriptions")]
    ChannelReadSubscriptions,
    /// Read the list of VIPs in your channel.
    #[static_str("channel:read:vips")]
    ChannelReadVips,
    /// Send live stream chat and rooms messages.
    #[static_str("chat:edit")]
    ChatEdit,
    /// View live stream chat and rooms messages.
    #[static_str("chat:read")]
    ChatRead,
    /// Manage Clips for a channel.
    #[static_str("clips:edit")]
    ClipsEdit,
    /// View a channel’s moderation data including Moderators, Bans, Timeouts, and Automod settings.
    #[static_str("moderation:read")]
    ModerationRead,
    /// Send announcements in channels where you have the moderator role.
    #[static_str("moderator:manage:announcements")]
    ModeratorManageAnnouncements,
    /// Manage messages held for review by AutoMod in channels where you are a moderator.
    #[static_str("moderator:manage:automod")]
    ModeratorManageAutoMod,
    /// Manage a broadcaster’s AutoMod settings
    #[static_str("moderator:manage:automod_settings")]
    ModeratorManageAutomodSettings,
    /// Ban and unban users.
    #[static_str("moderator:manage:banned_users")]
    ModeratorManageBannedUsers,
    /// Manage a broadcaster’s list of blocked terms.
    #[static_str("moderator:manage:blocked_terms")]
    ModeratorManageBlockedTerms,
    /// Delete chat messages in channels where you have the moderator role
    #[static_str("moderator:manage:chat_messages")]
    ModeratorManageChatMessages,
    /// View a broadcaster’s chat room settings.
    #[static_str("moderator:manage:chat_settings")]
    ModeratorManageChatSettings,
    /// Manage a broadcaster’s Shield Mode status.
    #[static_str("moderator:manage:shield_mode")]
    ModeratorManageShieldMode,
    /// Manage a broadcaster’s shoutouts.
    #[static_str("moderator:manage:shoutouts")]
    ModeratorManageShoutouts,
    /// View a broadcaster’s AutoMod settings.
    #[static_str("moderator:read:automod_settings")]
    ModeratorReadAutomodSettings,
    /// View a broadcaster’s list of blocked terms.
    #[static_str("moderator:read:blocked_terms")]
    ModeratorReadBlockedTerms,
    /// View a broadcaster’s chat room settings.
    #[static_str("moderator:read:chat_settings")]
    ModeratorReadChatSettings,
    /// View the chatters in a broadcaster’s chat room.
    #[static_str("moderator:read:chatters")]
    ModeratorReadChatters,
    /// Read the followers of a broadcaster.
    #[static_str("moderator:read:followers")]
    ModeratorReadFollowers,
    /// View a broadcaster’s Shield Mode status.
    #[static_str("moderator:read:shield_mode")]
    ModeratorReadShieldMode,
    /// View a broadcaster’s shoutouts.
    #[static_str("moderator:read:shoutouts")]
    ModeratorReadShoutouts,
    /// Allows client’s bot to act as this user.
    #[static_str("user:bot")]
    UserBot,
    /// Manage a user object.
    #[static_str("user:edit")]
    UserEdit,
    /// Edit your channel's broadcast configuration, including extension configuration. (This scope implies user:read:broadcast capability.)
    #[static_str("user:edit:broadcast")]
    UserEditBroadcast,
    /// Manage the block list of a user.
    #[static_str("user:manage:blocked_users")]
    UserManageBlockedUsers,
    /// Update the color used for the user’s name in chat.Update User Chat Color
    #[static_str("user:manage:chat_color")]
    UserManageChatColor,
    /// Read whispers that you send and receive, and send whispers on your behalf.
    #[static_str("user:manage:whispers")]
    UserManageWhispers,
    /// View the block list of a user.
    #[static_str("user:read:blocked_users")]
    UserReadBlockedUsers,
    /// View a user’s broadcasting configuration, including Extension configurations.
    #[static_str("user:read:broadcast")]
    UserReadBroadcast,
    /// View live stream chat and room messages.
    #[static_str("user:read:chat")]
    UserReadChat,
    /// View a user’s email address.
    #[static_str("user:read:email")]
    UserReadEmail,
    /// View the list of channels a user follows.
    #[static_str("user:read:follows")]
    UserReadFollows,
    /// Read the list of channels you have moderator privileges in.
    #[static_str("user:read:moderated_channels")]
    UserReadModeratedChannels,
    /// View if an authorized user is subscribed to specific channels.
    #[static_str("user:read:subscriptions")]
    UserReadSubscriptions,
    /// Send messages in a chat room.
    #[static_str("user:write:chat")]
    UserWriteChat,
    /// Send whisper messages.
    #[static_str("whispers:edit")]
    WhispersEdit,
    /// View your whisper messages.
    #[static_str("whispers:read")]
    WhispersRead,
}

#[derive(Clone, DerefNewType, NewTypeFrom)]
pub struct Scopes(#[deref(mut)] Vec<Scope>);

impl Scopes {
    pub fn as_string(&self) -> String {
        self.iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

impl FromIterator<Scope> for Scopes {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Scope>,
    {
        Self(iter.into_iter().collect())
    }
}

impl FromStr for Scopes {
    type Err = <Scope as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_whitespace()
            .map(Scope::from_str)
            .collect::<Result<Self, Self::Err>>()
    }
}

mod serde_impl {
    use serde::de::Visitor;
    use serde::{Deserialize, Serialize};

    use super::*;

    impl Serialize for Scopes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.as_string())
        }
    }

    struct ScopesVisitor;

    impl<'de> Visitor<'de> for ScopesVisitor {
        type Value = Scopes;

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            v.parse().or_else(|e| Err(E::custom(format!("{e:?}"))))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element()? {
                vec.push(elem);
            }
            Ok(vec.into())
        }

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "a string or sequence of strings that can be parsed by `<{} as FromStr>::from_str`",
                stringify!(Scopes)
            )
        }
    }

    impl<'de> Deserialize<'de> for Scopes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_any(ScopesVisitor)
        }
    }
}
