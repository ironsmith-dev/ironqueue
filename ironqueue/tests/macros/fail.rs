#[ironqueue::job(timeout_ms = "30000")]
async fn bad_duration(_: ()) {}

#[ironqueue::job(max_attempts = 2147483647)]
async fn bad_attempts(_: ()) {}

#[ironqueue::job(timeout_ms = 18446744073709551616)]
async fn overflowing_timeout(_: ()) {}

#[ironqueue::cron("* * * * *", revision = 9223372036854775808)]
async fn overflowing_revision() {}

#[ironqueue::job(max_backoff_ms = 1)]
async fn zero_delay_backoff(_: ()) {}

#[ironqueue::job(max_attempt = 3)]
async fn unknown_attribute(_: ()) {}

#[ironqueue::job(revision = 1)]
async fn job_with_cron_revision(_: ()) {}

#[ironqueue::job]
async fn no_payload() {}

#[ironqueue::job]
fn not_async(_: ()) {}

#[ironqueue::job]
async unsafe fn unsafe_job(_: ()) {}

#[ironqueue::cron("* * * * *")]
async unsafe fn unsafe_cron() {}

// `call()` forwards through a plain `async fn`, so it cannot carry an ABI — the
// handler's would be silently dropped while the user's own signature kept it.
#[ironqueue::job]
async extern "C" fn abi_job(_: ()) {}

#[ironqueue::job]
async fn generic<T: serde::de::DeserializeOwned>(args: T) {
    let _ = args;
}

#[ironqueue::job]
async fn impl_trait_job(_: impl serde::Serialize) {}

#[ironqueue::cron("* * * * *")]
async fn impl_trait_cron(_: impl Send) {}

#[ironqueue::job]
async fn impl_trait_return(_: ()) -> impl serde::Serialize {}

#[ironqueue::job]
async fn where_clause_only(args: u32)
where
    u32: Copy,
{
    let _ = args;
}

#[ironqueue::cron("99 * * * *")]
async fn impossible() {}

#[ironqueue::cron(30)]
async fn not_a_string() {}

#[derive(Clone)]
struct NotAnExtractor;

#[ironqueue::job]
async fn bad_extractor(_: (), value: NotAnExtractor) {
    let _ = value;
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Payload;

#[ironqueue::cron("* * * * *")]
async fn cron_payload(value: Payload) {
    let _ = value;
}

#[ironqueue::job]
async fn returns_a_bare_value(_: ()) -> u32 {
    1
}

#[ironqueue::job(timeout_ms = 3_153_600_000_001)]
async fn out_of_range_timeout(_: ()) {}

#[ironqueue::job(timeout_ms = 30u64)]
async fn suffixed_timeout(_: ()) {}

#[ironqueue::job(backoff)]
async fn removed_bare_backoff(_: ()) {}

// Not the last token, so `syn` hands the macro a unary negation rather than a
// negative literal, and the magnitude is one past what an `i16` holds.
#[ironqueue::job(priority = -32769, max_attempts = 2)]
async fn priority_below_the_minimum(_: ()) {}

#[ironqueue::job]
async fn variadic_job(_: (), _: ...) {}

// A lint level with nothing to name is not one the expansion can copy onto the
// items it writes, so it is left where the user put it for rustc to reject.
#[ironqueue::job]
#[expect]
async fn bare_expect(_: ()) {}

// A key with no negative encoding says so once, whichever way `syn` handed the
// sign over: folded into the literal when it is the attribute's last token, and
// left as a unary negation anywhere else. The two spellings used to be refused
// as "integer literal is out of range" and "expected an unsuffixed integer
// literal" — the same value, two messages, neither the reason.
#[ironqueue::job(max_attempts = -1)]
async fn negative_attempts_last(_: ()) {}

#[ironqueue::job(max_attempts = -1, timeout_ms = 5)]
async fn negative_attempts_first(_: ()) {}

#[ironqueue::cron("* * * * *", revision = -1)]
async fn negative_revision() {}

// A payload missing its `serde` derives has to say so on the payload type, the
// way the two `IntoJobResult` obligations land on the return type. The
// `DeserializeOwned` bound used to be spanned at the attribute instead, so one
// missing derive reported three errors pointing at two different places.
struct NotSerde;

#[ironqueue::job]
async fn payload_without_derives(_: NotSerde) {}

// An attribute macro runs before `cfg` is evaluated, so the expansion binds this
// parameter in every configuration while the handler it wraps only keeps it in
// one. The build that strips it used to fail with a bare arity error against
// `#[ironqueue::job]`, naming nothing that leads back to the `cfg`; refusing it
// here reports it like every other unsupported signature form. The error is the
// same whichever way the `cfg` evaluates, which is the point — the parameter
// cannot work in both configurations.
#[ironqueue::job]
async fn cfg_gated_parameter(_: (), #[cfg(any())] _metrics: u32) {}

// The `cfg_attr` form removes the parameter the same way, one evaluation
// later, so it is refused with the same diagnostic.
#[ironqueue::job]
async fn cfg_attr_gated_parameter(_: (), #[cfg_attr(any(), cfg(any()))] _metrics: u32) {}

// Parses, and no calendar ever matches it. Left to run, this reached
// `next_occurrence` as an `Error::Config`, which the worker classifies as a
// permanent rejection and disables the cron for the process's whole life — so a
// macro that already parses the expression refuses it here instead.
#[ironqueue::cron("0 0 30 2 *")]
async fn never_february_thirtieth() {}

#[ironqueue::cron("0 0 31 4 *")]
async fn never_april_thirty_first() {}

// The expansion binds these names as patterns while also emitting a unit struct
// named after the function, so a job named after one turned every *other* job in
// the module into a path-pattern error (E0530) spanned on the neighbour's
// attribute. Refused here, where the message can name the actual cause.
#[ironqueue::job]
async fn __config(_: ()) {}

#[ironqueue::job]
async fn __ctx(_: ()) {}

#[ironqueue::job]
async fn __arg0(_: ()) {}

// A borrowed payload declares no generic parameter when its lifetime is elided,
// so it passed the generics check and failed instead as "missing lifetime in
// associated type", suggesting a lifetime on an `impl` block the author cannot
// see.
#[ironqueue::job]
async fn borrowed_payload(_: &str) {}

#[ironqueue::job]
async fn borrowed_output(_: ()) -> &'static str {
    "borrowed"
}

async fn tick() {}

// Holding a non-`Send` value across an `.await` is the common handler mistake,
// and the erased future is the only place the expansion cannot avoid producing
// a diagnostic. Spanned on the handler body, "future created by async block is
// not `Send`" underlines the block that holds the `Rc` rather than
// `#[ironqueue::job]`, which named none of the user's tokens.
#[ironqueue::job]
async fn not_send_across_await(_: ()) {
    let local = ::std::rc::Rc::new(1u32);
    tick().await;
    drop(local);
}

fn main() {}
