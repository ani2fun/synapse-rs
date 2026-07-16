//! `/account` (oracle: `AccountPage`, the shared "account grammar" the admin panel reuses):
//! the identity card, then the danger zone — three permanent actions, each behind a styled
//! confirm modal, reporting through one inline status banner.

use leptos::prelude::*;
use synapse_shared::identity::MeDto;

use crate::identity::state::{self, ActionStatus, AuthStatus, AuthStore};

/// Which destructive action awaits confirmation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    Erase,
    EraseAll,
    Delete,
}

#[component]
pub fn AccountPage() -> impl IntoView {
    let auth = AuthStore::from_context();
    view! {
        <div class="account-page">
            <div class="account-page__inner">
                <h1 class="account-page__title">"Your account"</h1>
                {move || match auth.status.get() {
                    AuthStatus::Loading => view! { <p class="account-page__loading">"Loading…"</p> }.into_any(),
                    AuthStatus::Anonymous => view! {
                        <div class="account-page__identity account-page__identity--anon">
                            <p class="account-page__handle">"Not signed in"</p>
                            <p class="account-page__meta">"Sign in to run, submit, and manage your data."</p>
                            <button class="account-page__signin" on:click=move |_| auth.sign_in()>
                                "Sign in"
                            </button>
                        </div>
                    }
                    .into_any(),
                    AuthStatus::Authed(me) => signed_in(auth, &me).into_any(),
                }}
            </div>
        </div>
    }
}

fn signed_in(auth: AuthStore, me: &MeDto) -> impl IntoView + use<> {
    let status = RwSignal::new(ActionStatus::Idle);
    let pending: RwSignal<Option<Pending>> = RwSignal::new(None);
    let avatar = me
        .username
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    let handle = format!("@{}", me.username);
    let email = me.email.clone();
    view! {
        <div class="account-page__identity">
            <span class="account-page__avatar">{avatar}</span>
            <span class="account-page__id-text">
                <span class="account-page__handle">{handle}</span>
                {email.map(|e| view! { <span class="account-page__field">{e}</span> })}
                <span class="account-page__meta">"Signed in with Keycloak"</span>
            </span>
            <a class="account-page__back" href="/">"← Back to the library"</a>
        </div>
        <section class="account-page__danger">
            <p class="account-page__danger-head"><span class="account-page__danger-icon">"⚠"</span>" Danger zone"</p>
            <p class="account-page__danger-note">"These actions are permanent and can't be undone."</p>
            <StatusBanner status=status />
            {card(
                "Erase my submissions",
                "Permanently delete every code submission you've made, across all problems.",
                "Erase",
                pending,
                Pending::Erase,
            )}
            {card(
                "Erase all my data",
                "Erase your submissions and clear this browser's reading preferences. Reloads the page.",
                "Erase everything",
                pending,
                Pending::EraseAll,
            )}
            {card(
                "Delete my account",
                "Erase your submissions and permanently remove your sign-in. You'll be signed out.",
                "Delete account",
                pending,
                Pending::Delete,
            )}
        </section>
        <ConfirmModal auth=auth status=status pending=pending />
    }
}

fn card(
    title: &'static str,
    desc: &'static str,
    button: &'static str,
    pending: RwSignal<Option<Pending>>,
    action: Pending,
) -> impl IntoView + use<> {
    view! {
        <div class="account-page__card">
            <span class="account-page__card-text">
                <span class="account-page__card-title">{title}</span>
                <span class="account-page__card-desc">{desc}</span>
            </span>
            <button
                class="account-page__btn account-page__btn--danger"
                on:click=move |_| pending.set(Some(action))
            >
                <span class="account-page__btn-icon">"🗑"</span>
                {button}
            </button>
        </div>
    }
}

#[component]
fn StatusBanner(status: RwSignal<ActionStatus>) -> impl IntoView {
    view! {
        {move || {
            let (class, icon, message) = match status.get() {
                ActionStatus::Idle => return ().into_any(),
                ActionStatus::Busy(m) => ("account-page__status account-page__status--busy", "…", m),
                ActionStatus::Ok(m) => ("account-page__status account-page__status--ok", "✓", m),
                ActionStatus::Error(m) => ("account-page__status account-page__status--error", "✗", m),
            };
            view! {
                <p class=class><span class="account-page__status-icon">{icon}</span>" "{message}</p>
            }
            .into_any()
        }}
    }
}

/// The styled stand-in for `window.confirm` — Cancel or the named danger verb; Esc closes.
#[component]
fn ConfirmModal(
    auth: AuthStore,
    status: RwSignal<ActionStatus>,
    pending: RwSignal<Option<Pending>>,
) -> impl IntoView {
    let confirm = move |action: Pending| {
        pending.set(None);
        match action {
            Pending::Erase => state::erase_submissions(status),
            Pending::EraseAll => state::erase_all_data(status),
            Pending::Delete => state::delete_account(auth, status),
        }
    };
    view! {
        {move || {
            pending.get().map(|action| {
                let (title, body, verb) = match action {
                    Pending::Erase => (
                        "Erase your submissions?",
                        "Every attempt you've saved will be deleted. This can't be undone.",
                        "Erase",
                    ),
                    Pending::EraseAll => (
                        "Erase all your data?",
                        "Your submissions and this browser's reading preferences will be gone.",
                        "Erase everything",
                    ),
                    Pending::Delete => (
                        "Delete your account?",
                        "Your submissions and your sign-in will be permanently removed.",
                        "Delete account",
                    ),
                };
                view! {
                    <div
                        class="confirm-scrim"
                        on:click=move |event| {
                            if event.target() == event.current_target() {
                                pending.set(None);
                            }
                        }
                        on:keydown=move |event| {
                            if event.key() == "Escape" {
                                pending.set(None);
                            }
                        }
                    >
                        <div class="confirm">
                            <p class="confirm__title">{title}</p>
                            <p class="confirm__body">{body}</p>
                            <div class="confirm__actions">
                                <button class="confirm__cancel" on:click=move |_| pending.set(None)>
                                    "Cancel"
                                </button>
                                <button class="confirm__danger" on:click=move |_| confirm(action)>
                                    {verb}
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })
        }}
    }
}
