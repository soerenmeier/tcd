
const offsetLeftRight = 16;
const offsetTopBottom = 4;
const gapY = 8;
const gapX = 32;
export const width = 32;
export const height = 44;

const charsLineWidth = 8;

const charMap = [
	'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
	'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
	'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
	'Y', 'Z', '1', '2', '3', '4', '5', '6',
	'7', '8', '9', '0', 'a', '(', ')', '<',
	'>', '[', ']', '+', '-', '*', '/', '=',
	'o', '|', '{', '}', '.', ',', '!', '?',
	':', ';', '&', '_', '\'', '"', '%', '#',
	'@', ' '
];

export default class DedFont {
	constructor(file) {
		this.img = new Image;
		this.img.src = '/fonts/' + file;
	}

	async loaded() {
		return new Promise(resolve => {
			const loaded = () => {
				this.img.removeEventListener('load', loaded);
				resolve();
			}
			this.img.addEventListener('load', loaded);
		});
	}

	numToXY(num) {
		let x = num % charsLineWidth;
		x = offsetLeftRight + x * width + (x * gapX);
		let y = Math.floor(num / charsLineWidth);
		y = offsetTopBottom + y * height + (y * gapY);

		return { x, y };
	}

	draw(ctx, char, dx, dy, charWidth, charHeight) {
		let num = charMap.indexOf(char);
		if (num == -1)
			num = charMap.length;

		const { x, y } = this.numToXY(num);

		ctx.drawImage(
			this.img,
			x, y, width, height,
			dx, dy, charWidth, charHeight
		);
	}
}