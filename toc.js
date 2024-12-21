// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="intro.html"><strong aria-hidden="true">1.</strong> Intro</a></li><li class="chapter-item expanded affix "><li class="part-title">Usage</li><li class="chapter-item expanded "><a href="getting_started.html"><strong aria-hidden="true">2.</strong> Getting Started</a></li><li class="chapter-item expanded "><a href="template_macro.html"><strong aria-hidden="true">3.</strong> The Template Macro</a></li><li class="chapter-item expanded "><a href="language.html"><strong aria-hidden="true">4.</strong> Stilts Language</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="language/rust_expressions.html"><strong aria-hidden="true">4.1.</strong> Rust Expressions</a></li><li class="chapter-item expanded "><a href="language/control_expressions.html"><strong aria-hidden="true">4.2.</strong> Control Expressions</a></li><li class="chapter-item expanded "><a href="language/inheritance_expressions.html"><strong aria-hidden="true">4.3.</strong> Inheritance Expressions</a></li></ol></li><li class="chapter-item expanded "><a href="configuration.html"><strong aria-hidden="true">5.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="design_iteration.html"><strong aria-hidden="true">6.</strong> Design Iteration</a></li><li class="chapter-item expanded affix "><li class="part-title">Runtime Details</li><li class="chapter-item expanded "><a href="extension_traits.html"><strong aria-hidden="true">7.</strong> Extension Traits</a></li><li class="chapter-item expanded "><a href="performance.html"><strong aria-hidden="true">8.</strong> Performance</a></li><li class="chapter-item expanded affix "><li class="part-title">References</li><li class="chapter-item expanded "><a href="references.html"><strong aria-hidden="true">9.</strong> References</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
