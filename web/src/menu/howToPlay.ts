const guideTab = document.querySelector<HTMLButtonElement>('#guide-open')!;
const guide = document.querySelector<HTMLDialogElement>('#fight-guide')!;

guideTab.hidden = false;
guideTab.addEventListener('click', () => guide.showModal());
guide.addEventListener('close', () => guideTab.focus({preventScroll:true}));
