# Spike result: consuming the Rust repository from Swift & Kotlin

This spike built a fake `UserRepository` in Rust (`src/lib.rs`), then ran
UniFFI against the compiled `libspike_core.dylib` to generate **real** Swift and
Kotlin bindings (`generated/swift`, `generated/kotlin`). Everything below is
copied from the generated public surface — not hand-written.

The spike deliberately exercises the shapes a repository needs: value records,
plain enums, enums with associated data, `Option`, a typed error, a stateful
object, an async method, and a native-implemented observer callback.

## Rust definition (input)

```rust
#[derive(uniffi::Record)]
pub struct User { id: String, display_name: String, email: Option<String>, role: Role }

#[derive(uniffi::Enum)]
pub enum Role { Guest, Member, Admin }

#[derive(uniffi::Enum)]
pub enum SessionState {
    LoggedOut,
    Active { user: User, token: String },
    Expired { since_epoch_secs: u64 },
}

#[derive(uniffi::Error)]
pub enum RepoError { NotFound { id: String }, Network { reason: String } }

#[uniffi::export(with_foreign)]
pub trait SessionObserver: Send + Sync { fn on_session_changed(&self, state: SessionState); }

#[uniffi::export(async_runtime = "tokio")]
impl UserRepository {
    #[uniffi::constructor] pub fn new() -> Arc<Self> { /* ... */ }
    pub fn all_users(&self) -> Vec<User> { /* ... */ }
    pub fn user(&self, id: String) -> Result<User, RepoError> { /* ... */ }
    pub fn set_observer(&self, observer: Arc<dyn SessionObserver>) { /* ... */ }
    pub async fn fake_login(&self, id: String) -> Result<SessionState, RepoError> { /* ... */ }
    pub fn current_session(&self) -> SessionState { /* ... */ }
}
```

## Generated Swift (what iOS sees)

```swift
public struct User {
    public var id: String
    public var displayName: String   // snake_case -> camelCase automatically
    public var email: String?        // Option -> Optional
    public var role: Role
    public init(id: String, displayName: String, email: String?, role: Role) { ... }
}

public enum Role { case guest, member, admin }

public enum SessionState {
    case loggedOut
    case active(user: User, token: String)     // associated values
    case expired(sinceEpochSecs: UInt64)
}

extension RepoError: Foundation.LocalizedError { ... }   // real Swift Error

public protocol SessionObserver : AnyObject {
    func onSessionChanged(state: SessionState)
}

open class UserRepository {
    public convenience init()                                  // UserRepository()
    open func allUsers() -> [User]
    open func user(id: String) throws -> User                  // throws
    open func setObserver(observer: SessionObserver)
    open func fakeLogin(id: String) async throws -> SessionState   // async throws
    open func currentSession() -> SessionState
}
```

### iOS usage

```swift
final class LoginObserver: SessionObserver {
    func onSessionChanged(state: SessionState) {
        if case let .active(user, _) = state { print("logged in: \(user.displayName)") }
    }
}

let repo = UserRepository()
repo.setObserver(observer: LoginObserver())

let users = repo.allUsers()                       // [User]
let ada = try repo.user(id: "u_1")                // throws RepoError

Task {
    do {
        let session = try await repo.fakeLogin(id: "u_1")   // native async/await
        if case let .active(user, token) = session { ... }
    } catch RepoError.NotFound(let id) {
        print("no user \(id)")
    }
}
```

## Generated Kotlin (what Android sees)

```kotlin
data class User(
    var id: String,
    var displayName: String,   // snake_case -> camelCase automatically
    var email: String?,        // Option -> nullable
    var role: Role,
)

enum class Role { GUEST, MEMBER, ADMIN }

sealed class SessionState {
    object LoggedOut : SessionState()
    data class Active(val user: User, val token: String) : SessionState()
    data class Expired(val sinceEpochSecs: ULong) : SessionState()
}

sealed class RepoException : kotlin.Exception() {   // real Kotlin exception
    class NotFound(...) : RepoException()
    class Network(...) : RepoException()
}

interface SessionObserver { fun onSessionChanged(state: SessionState) }

open class UserRepository : Disposable, AutoCloseable {
    constructor()
    fun allUsers(): List<User>
    fun user(id: String): User                     // throws RepoException
    fun setObserver(observer: SessionObserver)
    suspend fun fakeLogin(id: String): SessionState   // suspend
    fun currentSession(): SessionState
}
```

### Android usage

```kotlin
val repo = UserRepository()
repo.setObserver(object : SessionObserver {
    override fun onSessionChanged(state: SessionState) {
        if (state is SessionState.Active) Log.d("session", state.user.displayName)
    }
})

val users: List<User> = repo.allUsers()

lifecycleScope.launch {
    try {
        when (val s = repo.fakeLogin("u_1")) {     // suspend, off the main thread
            is SessionState.Active   -> show(s.user, s.token)
            is SessionState.Expired  -> reauth()
            SessionState.LoggedOut   -> Unit
        }
    } catch (e: RepoException.NotFound) { /* ... */ }
}

repo.close()   // frees the Rust object (see notes below)
```

## What maps cleanly (we lose nothing here)

| Rust | Swift | Kotlin |
| --- | --- | --- |
| `struct` record | `struct` + memberwise init | `data class` (equals/copy/destructuring) |
| `enum` (no data) | `enum` | `enum class` |
| `enum` with data | `enum` with associated values + pattern match | `sealed class` + `when` exhaustive match |
| `Option<T>` | `T?` | `T?` (nullable) |
| `Result<T, E>` | `throws` + `Error` conformance | throwing fun + `Exception` subclass |
| `async fn` | `async throws` | `suspend fun` |
| `Vec<T>` | `[T]` | `List<T>` |
| `Arc<dyn Trait>` (foreign) | `protocol` you implement | `interface` you implement |
| snake_case | camelCase (auto) | camelCase (auto) |

Pattern matching, exhaustive `when`, native concurrency, and native error
handling all survive the boundary. This is the good news: **for the repository
/ use-case surface, idiomatic native code is preserved.**

## What we DO lose or must design around

1. **No Codable / Parcelable / Serializable for free.** Generated `struct`s and
   `data class`es are plain — they do not conform to `Codable` (Swift) or
   `Parcelable`/`kotlinx.serialization` (Kotlin). If a screen needs to persist
   a model in a `Bundle` or encode it to JSON on the native side, you add a
   native wrapper/mapper. Design answer: keep persistence in Rust (that is the
   architecture anyway — storage lives below the boundary), so native types stay
   transient view models.

2. **Objects are reference-counted handles, not value types.** `UserRepository`
   is a `class` backed by a Rust pointer. On Kotlin it is `AutoCloseable` and
   should be `.close()`d (or scoped) to free the Rust side promptly — the JVM
   GC will not do it deterministically. On Swift, ARC handles `deinit`. Records
   (`User`) are copied by value and need no cleanup. Design answer: hold
   long-lived objects (repositories, the session store) at app scope and close
   on teardown; pass records freely.

3. **No methods/computed properties on records.** A `uniffi::Record` is data
   only. `User.isAdmin` or a formatted display string is not carried across;
   put that logic either in Rust (exposed as a function) or in a tiny native
   extension. Enums likewise arrive without Rust `impl` methods.

4. **Callbacks cross a real thread boundary.** `SessionObserver.onSessionChanged`
   may fire on a Rust/tokio thread, not the main thread. Native code must hop to
   the main/UI dispatcher before touching UI. This is not a loss vs. any other
   async source, but it must be handled explicitly.

5. **No generics or trait objects beyond declared callbacks.** The exposed API
   is monomorphic — no `Repository<T>` across the boundary. Each concrete type
   is generated. For this app that is fine; it is a constraint to know.

6. **Enum/error naming quirks.** Kotlin renders a Rust error enum as
   `RepoException` (the `Error` suffix becomes `Exception`); associated-data
   error variants and default `equals` on some generated types have edges worth
   a lint pass. Cosmetic, not blocking.

## Verdict

The bridge generates cleanly for both platforms and preserves the native
patterns that matter for the View/ViewModel layers: value semantics for records,
sealed/associated-value enums with exhaustive matching, nullability, native
`async`/`suspend`, and native error handling. The real constraints are (a) no
free serialization/parcelization, (b) deterministic cleanup for Rust-backed
objects, and (c) no behavior (methods) on transferred data — all of which the
chosen architecture already accommodates by keeping storage and logic in Rust
and treating native models as transient.
