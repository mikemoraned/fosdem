function related(data) {
  const { nodes, links } = data;

  // Specify the dimensions of the chart.
  const width = 928;
  const height = 680;

  const distanceScale = 100;

  // Create a simulation with several forces.
  const simulation = d3
    .forceSimulation(nodes)
    .force(
      "link",
      d3
        .forceLink(links)
        .id((d) => d.index)
        .distance((d) => distanceScale * d.distance)
    )
    .force("charge", d3.forceManyBody())
    .force("x", d3.forceX())
    .force("y", d3.forceY());

  // Create the SVG container.
  const svg = d3
    .create("svg:svg")
    .attr("width", width)
    .attr("height", height)
    .attr("viewBox", [-width / 2, -height / 2, width, height])
    .attr("style", "max-width: 100%; height: auto;");

  // Add a line for each link, and a circle for each node.
  const link = svg
    .append("g")
    .attr("stroke", "#999")
    .attr("stroke-opacity", 0.6)
    .selectAll("line")
    .data(links)
    .join("line")
    .attr("stroke-width", (d) => d.distance);

  const node = svg
    .append("g")
    .attr("stroke", "#fff")
    .attr("stroke-width", 1.5)
    .selectAll("circle")
    .data(nodes)
    .join("circle")
    .attr("r", 5);
  // .attr("fill", (d) => color(d.group));

  node.append("title").text((d) => d.title);

  // Set the position attributes of links and nodes each time the simulation ticks.
  simulation.on("tick", () => {
    link
      .attr("x1", (d) => d.source.x)
      .attr("y1", (d) => d.source.y)
      .attr("x2", (d) => d.target.x)
      .attr("y2", (d) => d.target.y);

    node.attr("cx", (d) => d.x).attr("cy", (d) => d.y);
  });

  return svg.node();
}

console.log("Loading");
const data = await d3.json("/assets/all.json");
console.log(data);

const svg = related(data);
console.log(svg);
const container = document.getElementById("container");
container.append(svg);
