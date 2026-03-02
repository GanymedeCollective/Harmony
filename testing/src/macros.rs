//! DSL macros for integration tests.

/// Define a test world with platforms, users, and channel links.
///
/// ```ignore
/// test_world! {
///     platforms {
///         alpha: ["#test", "#general"],
///         beta: ["#test"],
///     }
///     users {
///         alice: { alpha: "4l1c3", beta: "Alice" },
///     }
///     channels {
///         alpha "#test" = beta "#test",
///         alpha "#general" = beta "#general",
///     }
/// }
/// ```
///
/// The `users` and `channels` sections are optional.
#[macro_export]
macro_rules! test_world {
    (
        platforms {
            $( $platform:ident : [ $($channel:literal),* $(,)? ] ),* $(,)?
        }
        $(users {
            $( $user:ident : {
                $($user_platform:ident : $user_name:literal),* $(,)?
            } ),* $(,)?
        })?
        $(channels {
            $( $($link_platform:ident $link_channel:literal)=+ ),* $(,)?
        })?
    ) => {{
        let mut __builder = $crate::TestWorld::builder();
        $(
            __builder = __builder.platform(
                stringify!($platform),
                &[ $($channel),* ],
            );
        )*
        $($(
            __builder = __builder.user(
                stringify!($user),
                &[ $( (stringify!($user_platform), $user_name) ),* ],
            );
        )*)?
        $($(
            __builder = __builder.link(
                &[ $( (stringify!($link_platform), $link_channel) ),+ ],
            );
        )*)?
        __builder.build()
    }};
}

/// Inject a message on a platform.
///
/// The author name is resolved through the test world's user identity map:
/// if `"alice"` is a known canonical name with an identity on that platform,
/// the platform-specific name is used; otherwise the string is used as-is.
///
/// ```ignore
/// send!(ctx, alpha, "alice", "#general", "hello");
/// ```
#[macro_export]
macro_rules! send {
    ($ctx:ident, $platform:ident, $author:expr, $channel:expr, $content:expr) => {
        $ctx.control(stringify!($platform))
            .inject_message($crate::Message {
                author: $crate::User {
                    id: None,
                    name: $ctx.resolve_author($author, stringify!($platform)),
                    display_name: None,
                    avatar_url: None,
                },
                channel: $crate::Channel {
                    id: $channel.to_owned(),
                    name: $channel.to_owned(),
                },
                content: $content.to_owned(),
                attachments: vec![],
            })
            .await
    };
}

/// Wait for the next relayed message on a platform and assert field values.
///
/// Field paths like `author.name` are supported.
///
/// ```ignore
/// expect!(ctx, beta, "#general", {
///     content == "hello",
///     author.name == "Alice",
/// });
/// ```
#[macro_export]
macro_rules! expect {
    ($ctx:ident, $platform:ident, $channel:expr, { $( $($field:ident).+ == $val:expr ),+ $(,)? }) => {{
        let (__ch, __msg) = $ctx
            .control(stringify!($platform))
            .next_message(::std::time::Duration::from_secs(2))
            .await
            .expect(concat!("expected ", stringify!($platform), " to receive a message"));
        assert_eq!(__ch.id, $channel);
        $(
            assert_eq!(__msg . $($field).+ , $val);
        )+
    }};
}

/// Assert that a platform received no message (200ms timeout).
///
/// ```ignore
/// expect_none!(ctx, gamma);
/// ```
#[macro_export]
macro_rules! expect_none {
    ($ctx:ident, $platform:ident) => {
        assert!(
            $ctx.control(stringify!($platform))
                .next_message(::std::time::Duration::from_millis(200))
                .await
                .is_none(),
            concat!(stringify!($platform), " should NOT receive any message"),
        );
    };
}
