(function() {var implementors = {};
implementors["egg"] = [{"text":"impl UnwindSafe for Id","synthetic":true,"types":[]},{"text":"impl UnwindSafe for ColorId","synthetic":true,"types":[]},{"text":"impl&lt;'a, L, N&gt; !UnwindSafe for Dot&lt;'a, L, N&gt;","synthetic":true,"types":[]},{"text":"impl&lt;L, D&gt; UnwindSafe for EClass&lt;L, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;D: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;L: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;L, N&gt; UnwindSafe for EGraph&lt;L, N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;N: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;N as Analysis&lt;L&gt;&gt;::Data: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;L as DiscriminantKind&gt;::Discriminant: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, CF, L, N&gt; !UnwindSafe for Extractor&lt;'a, CF, L, N&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for AstSize","synthetic":true,"types":[]},{"text":"impl UnwindSafe for AstDepth","synthetic":true,"types":[]},{"text":"impl&lt;L&gt; UnwindSafe for RecExpr&lt;L&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for SymbolLang","synthetic":true,"types":[]},{"text":"impl&lt;L&gt; UnwindSafe for Pattern&lt;L&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for SearchMatches","synthetic":true,"types":[]},{"text":"impl&lt;A1, A2&gt; UnwindSafe for ConditionEqual&lt;A1, A2&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A1: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;A2: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;C, A&gt; UnwindSafe for ConditionalApplier&lt;C, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;L, N&gt; !UnwindSafe for Rewrite&lt;L, N&gt;","synthetic":true,"types":[]},{"text":"impl&lt;L, N, IterData&nbsp;=&nbsp;()&gt; !UnwindSafe for Runner&lt;L, N, IterData&gt;","synthetic":true,"types":[]},{"text":"impl&lt;IterData&gt; UnwindSafe for Iteration&lt;IterData&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;IterData: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for SimpleScheduler","synthetic":true,"types":[]},{"text":"impl UnwindSafe for BackoffScheduler","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Subst","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Var","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Symbol","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Color","synthetic":true,"types":[]},{"text":"impl&lt;L&gt; UnwindSafe for ENodeOrVar&lt;L&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for StopReason","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()