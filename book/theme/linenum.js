document.addEventListener('DOMContentLoaded', () => {
    const els = document.querySelectorAll(".hljs.numbered");
    els.forEach(el => hljs.lineNumbersBlock(el));
});
