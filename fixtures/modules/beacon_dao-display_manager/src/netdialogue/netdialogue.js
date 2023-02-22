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

const nets = document.getElementsByClassName("netChoice");
let selected = #curr#;

buttons[1].addEventListener("click", () => {close(); impulse(address(), "do_change_network", selected)});
buttons[0].addEventListener("click", close);

Array.from(nets).forEach((n, i) => {
	n.addEventListener("mouseover", () => {
		n.style.opacity = "80%";
	});

	n.addEventListener("mouseout", () => {
		n.style.opacity = "100%";
	});

	n.addEventListener("click", () => {
		selected = i;

		Array.from(nets).forEach((n) => {
			n.style.fontWeight = "normal";
		});
		n.style.fontWeight = "bold";
	});
});
