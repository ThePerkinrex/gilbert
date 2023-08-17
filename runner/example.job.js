let a;
export default {
	params: [
		{
			name: "x",
			type: "number",
		},
		{
			name: "y",
			type: "number",
		},
	],
	stages: {
		stage1: (x, y) => {
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
	},
};
