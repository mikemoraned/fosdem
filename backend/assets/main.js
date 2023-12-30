function related(data) {
  // Specify the dimensions of the chart.
  const width = 928;
  const height = 680;

  // Create the SVG container.
  const svg = d3
    .create("svg:svg")
    .attr("width", width)
    .attr("height", height)
    .attr("viewBox", [-width / 2, -height / 2, width, height])
    .attr("style", "max-width: 100%; height: auto;");

  return svg.node();
}

console.log("Loading");
const data = await d3.json("/assets/all.json");
console.log(data);

const svg = related(data);
console.log(svg);
const container = document.getElementById("container");
container.append(svg);
