const container = document.getElementById("dialogueContainer");
container.style.opacity = "100%";

const buttons = [document.getElementById("leftButton"), document.getElementById("rightButton")];

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

buttons[0].addEventListener("click", () => {close(); impulse(address(), "system_dialogue_resp", 0, #cbid#)});
buttons[1].addEventListener("click", () => {close(); impulse(address(), "system_dialogue_resp", 1, #cbid#)});
