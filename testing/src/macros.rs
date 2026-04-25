//! DSL macros for integration tests.

/// Define a test world with platforms and users.
///
/// Channels with matching names across platforms are automatically linked
/// via auto-correlation.
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
/// }
/// ```
///
/// The `users` section is optional.
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
    ($ctx:ident, $platform:ident, $author:expr, $channel:expr, $content:expr) => {{
        let __author_id = $ctx.resolve_author($author, stringify!($platform));
        $ctx.control(stringify!($platform))
            .inject_message($crate::PlatformMessage {
                author: $crate::PlatformUser {
                    platform: $crate::PlatformId::new(stringify!($platform)),
                    id: __author_id.clone(),
                    display_name: Some(__author_id),
                    avatar_url: None,
                },
                channel: $crate::PlatformChannel {
                    platform: $crate::PlatformId::new(stringify!($platform)),
                    id: $channel.to_owned(),
                    name: $channel.to_owned(),
                },
                content: vec![$crate::PlatformMessageSegment::Text($content.to_owned())],
            })
            .await
    }};
}

/// Wait for the next relayed message on a platform and assert field values.
///
/// The channel assertion extracts the `PlatformChannel` for the receiving
/// platform from `CoreMessage.channel`.
///
/// ```ignore
/// expect!(ctx, beta, "#general", {
///     content == "hello",
/// });
/// ```
#[macro_export]
macro_rules! expect {
    // Special arm: content == "..." renders the rope as plain text before comparing.
    ($ctx:ident, $platform:ident, $channel:expr, { content == $val:expr $(,)? }) => {{
        let __msg = $ctx
            .control(stringify!($platform))
            .next_message(::std::time::Duration::from_secs(2))
            .await
            .expect(concat!("expected ", stringify!($platform), " to receive a message"));
        let __pc = __msg.channel
            .get_platform_channel(&$crate::PlatformId::new(stringify!($platform)))
            .expect(concat!("message should have channel alias for ", stringify!($platform)));
        assert_eq!(__pc.id, $channel);
        assert_eq!($crate::rope_to_text(&__msg.content), $val);
    }};
    // Generic arm: compare fields directly.
    ($ctx:ident, $platform:ident, $channel:expr, { $( $($field:ident).+ == $val:expr ),+ $(,)? }) => {{
        let __msg = $ctx
            .control(stringify!($platform))
            .next_message(::std::time::Duration::from_secs(2))
            .await
            .expect(concat!("expected ", stringify!($platform), " to receive a message"));
        let __pc = __msg.channel
            .get_platform_channel(&$crate::PlatformId::new(stringify!($platform)))
            .expect(concat!("message should have channel alias for ", stringify!($platform)));
        assert_eq!(__pc.id, $channel);
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
