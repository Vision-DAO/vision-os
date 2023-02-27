const dateLabel = document.getElementById("dateDisplay");
const timeLabel = document.getElementById("timeDisplay");

const setDate = () => {
	const now = new Date();
	dateLabel.innerText = now.toLocaleString("default", { month: "long", day: "numeric", year: "numeric"});
	timeLabel.innerText = now.toLocaleString("default", { hour: "numeric", minute: "numeric" });
};

setDate();

setInterval(setDate, 1000);

const netButton = document.getElementById("networkSelector");

netButton.addEventListener("mouseover", () => {
	netButton.style.opacity = "60%";
});

netButton.addEventListener("mouseout", () => {
	netButton.style.opacity = "100%";
});

netButton.addEventListener("click", () => impulse(address(), "change_network", 0));

const ipfsButton = document.getElementById("ipfsSelector");

ipfsButton.addEventListener("mouseover", () => {
	ipfsButton.style.opacity = "60%";
});

ipfsButton.addEventListener("mouseout", () => {
	ipfsButton.style.opacity = "100%";
});

ipfsButton.addEventListener("click", () => impulse(address(), "change_ipfs_endpoint", 0));
