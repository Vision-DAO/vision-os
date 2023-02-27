const container = document.getElementById("netdialogueContainer");
container.style.opacity = "0%";

setTimeout(() => {
	container.style.opacity = "100%";
}, 100);

const buttons = [document.getElementById("leftButtonNet"), document.getElementById("rightButtonNet")];

buttons.forEach((b) => {
	b.addEventListener("mouseover", () => {
		b.style.opacity = "80%";
	});

	b.addEventListener("mouseout", () => {
		b.style.opacity = "100%";
	});
});

const close = () => {
	container.style.opacity = "0%";

	setTimeout(() => {
		container.remove();
	}, 300);
};

const input = document.getElementById("endpointInput");

buttons[1].addEventListener("click", () => {close(); impulse(address(), "do_change_endpoint", input.value)});
buttons[0].addEventListener("click", close);

