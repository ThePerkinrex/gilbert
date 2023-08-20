
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
			console.log(a)
		},
		final_stage: async () => {
			const data = await fetch("http://example.com").then(t => t.text())
			return data;
		},
};
