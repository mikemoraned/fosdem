function createSimulation(nodes, links, distanceScale) {
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

  return simulation;
}

function vis(data, initMaxDistance) {
  const { nodes, links } = data;

  const width = 928;
  const height = 680;

  const distanceScale = 100;

  const filteredLinks = links.filter((d) => d.distance <= initMaxDistance);
  var simulation = createSimulation(nodes, filteredLinks, distanceScale);

  // Create the SVG container.
  const svg = d3
    .create("svg:svg")
    .attr("width", width)
    .attr("height", height)
    .attr("viewBox", [-width / 2, -height / 2, width, height])
    .attr("style", "max-width: 100%; height: auto;");

  const linkSelection = svg
    .append("g")
    .attr("stroke", "#999")
    .attr("stroke-opacity", 0.6)
    .selectAll("line")
    .data(links)
    .join("line")
    .attr("stroke-width", (d) => d.distance);

  const nodeSelection = svg
    .append("g")
    .attr("stroke", "#fff")
    .attr("stroke-width", 1.5)
    .selectAll("circle")
    .data(nodes)
    .join("circle")
    .attr("r", 5);

  function tick(simulation, linkSelection, nodeSelection) {
    simulation.on("tick", () => {
      linkSelection
        .attr("x1", (d) => d.source.x)
        .attr("y1", (d) => d.source.y)
        .attr("x2", (d) => d.target.x)
        .attr("y2", (d) => d.target.y);

      nodeSelection.attr("cx", (d) => d.x).attr("cy", (d) => d.y);
    });
  }

  tick(simulation, linkSelection, nodeSelection);

  nodeSelection.append("title").text((d) => d.title);

  function distanceControl(maxDistance) {
    console.log(maxDistance);
    const filteredLinks = links.filter((d) => d.distance <= maxDistance);
    console.dir(filteredLinks);
    simulation.stop();

    linkSelection.data(filteredLinks);
    simulation = createSimulation(nodes, filteredLinks, distanceScale);
    tick(simulation, linkSelection, nodeSelection);
  }

  return [svg.node(), distanceControl];
}

console.log("Loading");
const data = await d3.json("/assets/all.limit2.json");
console.log(data);

const containerElement = document.getElementById("container");
const distanceFilterElement = document.querySelector(
  "#controls input.distance_filter"
);

const currentMaxDistance = distanceFilterElement.value;
const [svgElement, distanceControlFn] = vis(data, currentMaxDistance);
containerElement.append(svgElement);
distanceFilterElement.addEventListener("input", (e) => {
  distanceControlFn(distanceFilterElement.value);
});
