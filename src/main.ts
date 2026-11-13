import "./vendor/peep.js";

const root = document.querySelector<HTMLElement>("#sessions")!;
root.innerHTML = `<peep-bot state="idle" color="yellow" size="64"></peep-bot>`;
