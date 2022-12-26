const btn = document.getElementById("countBtn");

btn.addEventListener("click", (e) => {
	impulse(address(), "bump", []);
});
