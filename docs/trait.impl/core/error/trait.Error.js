(function() {var implementors = {
"ciborium":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"ciborium/value/enum.Error.html\" title=\"enum ciborium::value::Error\">Error</a>"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"ciborium/de/enum.Error.html\" title=\"enum ciborium::de::Error\">Error</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"ciborium/ser/enum.Error.html\" title=\"enum ciborium::ser::Error\">Error</a>&lt;T&gt;"]],
"clap_builder":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"clap_builder/parser/enum.MatchesError.html\" title=\"enum clap_builder::parser::MatchesError\">MatchesError</a>"],["impl&lt;F: <a class=\"trait\" href=\"clap_builder/error/trait.ErrorFormatter.html\" title=\"trait clap_builder::error::ErrorFormatter\">ErrorFormatter</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"clap_builder/error/struct.Error.html\" title=\"struct clap_builder::error::Error\">Error</a>&lt;F&gt;"]],
"criterion_plot":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"criterion_plot/enum.VersionError.html\" title=\"enum criterion_plot::VersionError\">VersionError</a>"]],
"either":[["impl&lt;L, R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"either/enum.Either.html\" title=\"enum either::Either\">Either</a>&lt;L, R&gt;<div class=\"where\">where\n    L: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>,\n    R: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>,</div>"]],
"getrandom":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"getrandom/struct.Error.html\" title=\"struct getrandom::Error\">Error</a>"]],
"itertools":[["impl&lt;I&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"itertools/structs/struct.ExactlyOneError.html\" title=\"struct itertools::structs::ExactlyOneError\">ExactlyOneError</a>&lt;I&gt;<div class=\"where\">where\n    I: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,\n    I::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#associatedtype.Item\" title=\"type core::iter::traits::iterator::Iterator::Item\">Item</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</div>"]],
"plotters":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"plotters/drawing/struct.DummyBackendError.html\" title=\"struct plotters::drawing::DummyBackendError\">DummyBackendError</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"plotters/drawing/enum.DrawingAreaErrorKind.html\" title=\"enum plotters::drawing::DrawingAreaErrorKind\">DrawingAreaErrorKind</a>&lt;E&gt;"]],
"plotters_backend":[["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"plotters_backend/enum.DrawingErrorKind.html\" title=\"enum plotters_backend::DrawingErrorKind\">DrawingErrorKind</a>&lt;E&gt;"]],
"proc_macro2":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"proc_macro2/struct.LexError.html\" title=\"struct proc_macro2::LexError\">LexError</a>"]],
"rand":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"rand/distributions/enum.BernoulliError.html\" title=\"enum rand::distributions::BernoulliError\">BernoulliError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"rand/distributions/enum.WeightedError.html\" title=\"enum rand::distributions::WeightedError\">WeightedError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"rand/rngs/adapter/struct.ReadError.html\" title=\"struct rand::rngs::adapter::ReadError\">ReadError</a>"]],
"rand_core":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"rand_core/struct.Error.html\" title=\"struct rand_core::Error\">Error</a>"]],
"rayon_core":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"rayon_core/struct.ThreadPoolBuildError.html\" title=\"struct rayon_core::ThreadPoolBuildError\">ThreadPoolBuildError</a>"]],
"regex":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"regex/enum.Error.html\" title=\"enum regex::Error\">Error</a>"]],
"regex_automata":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/meta/struct.BuildError.html\" title=\"struct regex_automata::meta::BuildError\">BuildError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/nfa/thompson/struct.BuildError.html\" title=\"struct regex_automata::nfa::thompson::BuildError\">BuildError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/struct.MatchError.html\" title=\"struct regex_automata::MatchError\">MatchError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/struct.PatternSetInsertError.html\" title=\"struct regex_automata::PatternSetInsertError\">PatternSetInsertError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/util/captures/struct.GroupInfoError.html\" title=\"struct regex_automata::util::captures::GroupInfoError\">GroupInfoError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/util/look/struct.UnicodeWordBoundaryError.html\" title=\"struct regex_automata::util::look::UnicodeWordBoundaryError\">UnicodeWordBoundaryError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/util/primitives/struct.PatternIDError.html\" title=\"struct regex_automata::util::primitives::PatternIDError\">PatternIDError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/util/primitives/struct.SmallIndexError.html\" title=\"struct regex_automata::util::primitives::SmallIndexError\">SmallIndexError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/util/primitives/struct.StateIDError.html\" title=\"struct regex_automata::util::primitives::StateIDError\">StateIDError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/util/wire/struct.DeserializeError.html\" title=\"struct regex_automata::util::wire::DeserializeError\">DeserializeError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_automata/util/wire/struct.SerializeError.html\" title=\"struct regex_automata::util::wire::SerializeError\">SerializeError</a>"]],
"regex_syntax":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"regex_syntax/enum.Error.html\" title=\"enum regex_syntax::Error\">Error</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_syntax/ast/struct.Error.html\" title=\"struct regex_syntax::ast::Error\">Error</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_syntax/hir/struct.CaseFoldError.html\" title=\"struct regex_syntax::hir::CaseFoldError\">CaseFoldError</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_syntax/hir/struct.Error.html\" title=\"struct regex_syntax::hir::Error\">Error</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"regex_syntax/struct.UnicodeWordError.html\" title=\"struct regex_syntax::UnicodeWordError\">UnicodeWordError</a>"]],
"serde":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"serde/de/value/struct.Error.html\" title=\"struct serde::de::value::Error\">Error</a>"]],
"serde_json":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"serde_json/struct.Error.html\" title=\"struct serde_json::Error\">Error</a>"]],
"syn":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"syn/parse/struct.Error.html\" title=\"struct syn::parse::Error\">Error</a>"]],
"tinytemplate":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"tinytemplate/error/enum.Error.html\" title=\"enum tinytemplate::error::Error\">Error</a>"]],
"walkdir":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"struct\" href=\"walkdir/struct.Error.html\" title=\"struct walkdir::Error\">Error</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()