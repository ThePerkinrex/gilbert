let a;
export default {
		stage1: (x, y) => {
			a = x + y;
		},
		stage2: () => {
			a += 2;
		},
		stage0: () => {
			console.log("yes");
		},
		final_stage: async () => {
			const data = await fetch("http://example.com").then(t => t.text())
			return data;
		},
};
