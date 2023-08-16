let a
export default {
	"stage1": (x,y) => {
		a = x+y
	},
	"stage2": () => {
		a+=2
	},
	"final_stage": () => {
		return a
	}
}