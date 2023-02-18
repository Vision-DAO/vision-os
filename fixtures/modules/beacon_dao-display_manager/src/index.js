const clickers = document.getElementById("loginProfiles");
const nextContainer = document.getElementById("loginField");
const back = document.getElementById("backBtn");
const loginBtn = document.getElementById("loginBtn");
const loginAddrInput = document.getElementById("loginAddr");

const guest = document.getElementById("loginProfile");

const next = () => {
	clickers.style.opacity = "0%";
	setTimeout(() => {
		clickers.style.display = "none";
		nextContainer.style.display = "flex";
		nextContainer.style.opacity = "100%";
	}, 300);
};

const prev = () => {
	nextContainer.style.opacity = "0%";
	setTimeout(() => {
		nextContainer.style.display = "none";
		clickers.style.display = "flex";
		clickers.style.opacity = "100%";
	}, 300);
};

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

// Show the net page when the login button is clicked
login.addEventListener("click", next);

// When the guest profile is activated, load its DAO modules
guest.addEventListener("click", () => {
	
});

back.addEventListener("click", prev);
back.addEventListener("mouseover", () => {
	back.style.opacity = "60%";
});
back.addEventListener("mouseout", () => {
	back.style.opacity = "100%";
});

loginBtn.addEventListener("mouseover", () => {
	loginBtn.style.opacity = "60%";
});

loginBtn.addEventListener("mouseout", () => {
	loginBtn.style.opacity = "100%";
});

loginBtn.addEventListener("click", () => {
	impulse(address(), "login_as", loginAddrInput.value);
});
