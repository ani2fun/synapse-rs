//! `/admin` (oracle: `AdminPage`, step 35) — the allowlist panel on the account grammar: the
//! grants table, the grant form, revoke per row, one status banner. `MeDto.admin` only gates
//! the UI; every call is re-checked server-side, so a non-admin who navigates here just sees
//! the API's 403 in the banner.

use leptos::prelude::*;
use leptos::task::spawn_local;
use synapse_shared::submission::{AllowlistEntryDto, GrantRequestDto};

use crate::api::{self, AsyncResult};
use crate::identity::state::{ActionStatus, AuthStatus, AuthStore};

#[component]
pub fn AdminPage() -> impl IntoView {
    let auth = AuthStore::from_context();
    view! {
        <div class="account-page">
            <div class="account-page__inner">
                <h1 class="account-page__title">"Admin — submit allowlist"</h1>
                {move || match auth.status.get() {
                    AuthStatus::Loading => view! { <p class="account-page__loading">"Loading…"</p> }.into_any(),
                    AuthStatus::Anonymous => view! {
                        <div class="account-page__identity account-page__identity--anon">
                            <p class="account-page__handle">"Not signed in"</p>
                            <button class="account-page__signin" on:click=move |_| auth.sign_in()>
                                "Sign in"
                            </button>
                        </div>
                    }
                    .into_any(),
                    AuthStatus::Authed(me) if !me.admin => view! {
                        <div class="account-page__identity account-page__identity--anon">
                            <p class="account-page__handle">"Admin only"</p>
                            <p class="account-page__meta">
                                "This deployment doesn't list you as an admin."
                            </p>
                        </div>
                    }
                    .into_any(),
                    AuthStatus::Authed(_) => panel().into_any(),
                }}
            </div>
        </div>
    }
}

fn panel() -> impl IntoView + use<> {
    let status = RwSignal::new(ActionStatus::Idle);
    let entries: RwSignal<AsyncResult<Vec<AllowlistEntryDto>>> = RwSignal::new(AsyncResult::Loading);
    let username = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());

    let reload = move || {
        spawn_local(async move {
            match api::allowlist().await {
                Ok(rows) => entries.set(AsyncResult::Loaded(rows)),
                Err(message) => entries.set(AsyncResult::Failed(message)),
            }
        });
    };
    reload();

    let grant = move || {
        let request = GrantRequestDto {
            username: username.get_untracked(),
            note: Some(note.get_untracked()).filter(|n| !n.trim().is_empty()),
        };
        if request.username.trim().is_empty() {
            return status.set(ActionStatus::Error("A grant needs a username".into()));
        }
        status.set(ActionStatus::Busy("Granting…".into()));
        spawn_local(async move {
            match api::allowlist_grant(&request).await {
                Ok(entry) => {
                    status.set(ActionStatus::Ok(format!("Granted '{}'.", entry.username)));
                    username.set(String::new());
                    note.set(String::new());
                    reload();
                }
                Err(message) => status.set(ActionStatus::Error(message)),
            }
        });
    };

    let revoke = move |name: String| {
        status.set(ActionStatus::Busy(format!("Revoking '{name}'…")));
        spawn_local(async move {
            match api::allowlist_revoke(&name).await {
                Ok(()) => {
                    status.set(ActionStatus::Ok(format!("Revoked '{name}'.")));
                    reload();
                }
                Err(message) => status.set(ActionStatus::Error(message)),
            }
        });
    };

    view! {
        <p class="account-page__meta">
            "Who may SAVE attempts when the allowlist is enforced. Usernames are stored lowercase."
        </p>
        <StatusBanner status=status />
        <form
            class="admin__grant"
            on:submit=move |event| {
                event.prevent_default();
                grant();
            }
        >
            <input
                class="admin__input"
                placeholder="username"
                prop:value=move || username.get()
                on:input=move |event| username.set(event_target_value(&event))
            />
            <input
                class="admin__input admin__input--note"
                placeholder="note (optional)"
                prop:value=move || note.get()
                on:input=move |event| note.set(event_target_value(&event))
            />
            <button class="admin__grant-btn" type="submit">"Grant"</button>
        </form>
        {move || match entries.get() {
            AsyncResult::Loading => view! { <p class="account-page__loading">"Loading grants…"</p> }.into_any(),
            AsyncResult::Failed(message) => {
                view! { <p class="account-page__status account-page__status--error">{message}</p> }.into_any()
            }
            AsyncResult::Loaded(rows) if rows.is_empty() => {
                view! { <p class="account-page__meta">"No grants yet."</p> }.into_any()
            }
            AsyncResult::Loaded(rows) => view! {
                <table class="admin__table">
                    <thead>
                        <tr><th>"Username"</th><th>"Note"</th><th>"Granted"</th><th></th></tr>
                    </thead>
                    <tbody>
                        {rows.into_iter().map(|entry| row(&entry, revoke)).collect::<Vec<_>>()}
                    </tbody>
                </table>
            }
            .into_any(),
        }}
    }
}

fn row<R: Fn(String) + Copy + 'static>(entry: &AllowlistEntryDto, revoke: R) -> impl IntoView + use<R> {
    let name = entry.username.clone();
    let revoke_name = name.clone();
    let note = entry.note.clone().unwrap_or_default();
    let granted = entry.granted_at.split('T').next().unwrap_or_default().to_owned();
    view! {
        <tr>
            <td class="admin__cell-user">{name}</td>
            <td>{note}</td>
            <td>{granted}</td>
            <td>
                <button
                    class="admin__revoke"
                    on:click=move |_| revoke(revoke_name.clone())
                >
                    "Revoke"
                </button>
            </td>
        </tr>
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
