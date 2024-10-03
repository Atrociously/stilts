# Performance

While performance is not a current target for improvement in Stilts, it does perform well when compared to other rust template engines.
The tests ran were modified and updated from these [template benchmarks](https://github.com/rosetta-rs/template-benchmarks-rs), which have not been
updated in some time. The benchmark code will be released to open source soon to provide better insight into methodology, but it hasn't changed much
from the linked benchmarks.

Stilts underperforms slightly when compared to other compiled template engines, but it still greatly out performs runtime engines.
This result is at least partially expected, other compiled template engines are able to employ certain optimizations at compile
time, that have not been implemented in Stilts.

<boxit key="big_table"></boxit>

<boxit key="teams"></boxit>
