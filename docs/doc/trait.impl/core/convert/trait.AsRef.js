(function() {
    var implementors = Object.fromEntries([["arrayvec",[["impl&lt;T, const CAP: usize&gt; AsRef&lt;[T]&gt; for <a class=\"struct\" href=\"arrayvec/struct.ArrayVec.html\" title=\"struct arrayvec::ArrayVec\">ArrayVec</a>&lt;T, CAP&gt;"],["impl&lt;const CAP: usize&gt; AsRef&lt;str&gt; for <a class=\"struct\" href=\"arrayvec/struct.ArrayString.html\" title=\"struct arrayvec::ArrayString\">ArrayString</a>&lt;CAP&gt;"]]],["zeroize",[["impl&lt;T, Z&gt; AsRef&lt;T&gt; for <a class=\"struct\" href=\"zeroize/struct.Zeroizing.html\" title=\"struct zeroize::Zeroizing\">Zeroizing</a>&lt;Z&gt;<div class=\"where\">where\n    T: ?Sized,\n    Z: AsRef&lt;T&gt; + <a class=\"trait\" href=\"zeroize/trait.Zeroize.html\" title=\"trait zeroize::Zeroize\">Zeroize</a>,</div>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[375,346]}