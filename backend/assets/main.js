function createSimulation(nodes, links, distanceScale, dimensions) {
  const { width, height } = dimensions;
  const maxGroup = Math.max(...nodes.map((d) => d.group));
  console.log("max group", maxGroup);
  const numGroups = maxGroup + 1;

  const simulation = d3
    .forceSimulation(nodes)
    .force(
      "link",
      d3
        .forceLink(links)
        .id((d) => d.index)
        .distance((d) => distanceScale * d.distance)
    )
    .force("charge", d3.forceManyBody().strength(-20))
    // .force(
    //   "x",
    //   d3.forceX().x((d) => {
    //     const stride = width / numGroups;
    //     const offset = stride * d.group;
    //     return offset;
    //   })
    // )
    .force("x", d3.forceX())
    .force("y", d3.forceY());

  return simulation;
}

function openLink(node) {
  window.open(node.url, "_blank");
}

function vis(data, initMinDistance, initMaxDistance) {
  const { nodes, links } = data;
  // for now, fake the group by assigning each to a group arbitrarily
  const numGroups = 20;
  nodes.forEach((d) => {
    d.group = d.index % numGroups;
  });

  const width = 928;
  const height = 680;
  const dimensions = { width, height };

  const distanceScale = 100;

  function filterLinks(maxDistance) {
    return links.filter((d) => d.distance <= maxDistance);
  }

  const filteredLinks = filterLinks(initMaxDistance);
  var simulation = createSimulation(
    nodes,
    filteredLinks,
    distanceScale,
    dimensions
  );

  const color = d3.scaleOrdinal(d3.schemeTableau10);

  // Create the SVG container.
  const svg = d3
    .create("svg:svg")
    .attr("width", width)
    .attr("height", height)
    .attr("viewBox", [-width / 2, -height / 2, width, height])
    .attr("style", "max-width: 100%; height: auto;");

  // a circle at 0,0, for debugging
  svg
    .append("g")
    .append("circle")
    .attr("cx", 0)
    .attr("cy", 0)
    .attr("fill", "black")
    .attr("r", 7);

  const linkSelection = svg
    .append("g")
    .attr("stroke", "#999")
    .attr("stroke-opacity", 0.6)
    .selectAll("line")
    .data(links)
    .join("line")
    .attr("stroke-width", (d) => 0.1 * distanceScale * d.distance);

  const nodeSelection = svg
    .append("g")
    .attr("stroke", "#fff")
    .attr("stroke-width", 1.5)
    .selectAll("circle")
    .data(nodes)
    .join("circle")
    .attr("fill", (d) => color(d.group))
    .attr("r", 4);

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

  linkSelection.append("title").text((d) => d.distance);
  nodeSelection.append("title").text((d) => d.title);
  nodeSelection.on("click", (e) => openLink(e.target.__data__));

  function distanceControl(minDistance, maxDistance) {
    const minSeparation = 0.01;
    const clampedMinDistance =
      minDistance + minSeparation <= maxDistance
        ? minDistance
        : maxDistance - minSeparation;
    const clampedMaxDistance =
      clampedMinDistance + minSeparation <= maxDistance
        ? maxDistance
        : clampedMinDistance + minSeparation;

    console.log(`${clampedMinDistance} -> ${clampedMaxDistance}`);
    const filteredLinks = filterLinks(clampedMaxDistance);
    const clusteredLinks = filteredLinks.map((d) => {
      var rolledUp = {
        ...d,
        distance: 0.05,
      };
      if (d.distance <= clampedMinDistance) {
        return rolledUp;
      } else {
        return d;
      }
    });
    simulation.stop();

    linkSelection.data(clusteredLinks);
    simulation = createSimulation(
      nodes,
      clusteredLinks,
      distanceScale,
      dimensions
    );
    tick(simulation, linkSelection, nodeSelection);

    return [clampedMinDistance, clampedMaxDistance];
  }

  return [svg.node(), distanceControl];
}

console.log("Loading");
const data = await d3.json("/assets/all.limit5.json");
console.log(data);

const containerElement = document.getElementById("container");
const minDistanceFilterElement = document.querySelector(
  "#controls input.min_distance_filter"
);
const maxDistanceFilterElement = document.querySelector(
  "#controls input.max_distance_filter"
);

const currentMinDistance = minDistanceFilterElement.value;
const currentMaxDistance = maxDistanceFilterElement.value;
const [svgElement, distanceControlFn] = vis(
  data,
  currentMinDistance,
  currentMaxDistance
);
containerElement.append(svgElement);
function handleChange() {
  const [clampedMinDistance, clampedMaxDistance] = distanceControlFn(
    minDistanceFilterElement.value,
    maxDistanceFilterElement.value
  );
  minDistanceFilterElement.value = clampedMinDistance;
  maxDistanceFilterElement.value = clampedMaxDistance;
}
minDistanceFilterElement.addEventListener("input", handleChange);
maxDistanceFilterElement.addEventListener("input", handleChange);
