const guest = document.getElementById("loginProfile");

guest.addEventListener("mouseover", () => {
	guest.style.opacity = "60%";
});

guest.addEventListener("mouseout", () => {
	guest.style.opacity = "100%";
});

const login = document.getElementById("loginBtnContainer");

login.addEventListener("mouseover", () => {
	login.style.opacity = "60%";
});

login.addEventListener("mouseout", () => {
	login.style.opacity = "100%";
});

// When the guest profile is activated, load its DAO modules
guest.addEventListener("click", () => {
	
});
