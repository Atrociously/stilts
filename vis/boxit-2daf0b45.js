function toUpperWords(str) {
    return str.split(" ")
        .map(word => word[0].toUpperCase() + word.substr(1))
        .join(" ");
}

function scaleUnits(benches) {
    const max = benches.flatMap(x => x.data)
        .reduce((acc, v) => v > acc ? v : acc, 0);

    let measure;
    let factor;
    if (max / 1e3 <= 10) {
        // units are microseconds
        measure = "μs"
        factor = 1e3;
    } else if (max / 1e6 <= 10) {
        // units are milliseconds
        measure = "ms";
        factor = 1e6;
    } else if (max / 1e9 <= 10) {
        // units are seconds
        measure = "s";
        factor = 1e9;
    }

    return [
        measure,
        benches.map(x => ({name: x.name.replace("Bench", ""), x: x.data.map(v => v / factor), type: 'box'}))
    ];
}

const dataPromise = fetch("https://raw.githubusercontent.com/Atrociously/stilts/refs/heads/master/book/vis/data.json").then(res => res.ok ? res.json() : {});
document.addEventListener('DOMContentLoaded', async () => {
    const plotEls = document.getElementsByTagName("boxit");
    if (plotEls.length === 0) {return;}

    let data = await dataPromise;
    for (el of plotEls) {
        const key = el.getAttribute("key");
        if (!key) {return;}

        const title = toUpperWords(key.replace('_', ' '));

        const benches = data[key];
        const [measure, traces] = scaleUnits(benches);
        Plotly.newPlot(el, traces, {
            title: title,
            xaxis: {
                title: {
                    text: `Time (${measure})`
                }
            }
        });
    }
}, false);
