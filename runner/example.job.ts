
let a: number;
export default  {
		stage1: (x: number, y: number) => {
			a = x + y;
		},
		stage2: () => {
			a += 2;
		},
		stage0: () => {
			console.log("yes");
		},
		final_stage: () => {
			return a;
		},
};
