export interface Job<Params extends ParamType[]> {
	params: ParamsList<Params>,
	stages: {
		[name: string]: Stage<Params>
	}
}

type ParamType = number | string | object
type ParamTypeAsName<T extends ParamType> = T extends number ? "number" : (T extends string ? "string" : "object")

interface Param<T extends ParamType> {
	name: string,
	type: ParamTypeAsName<T>
}

type ParamsList<P extends ParamType[]> = {
	[K in keyof P]: Param<P[K]>
}

interface Stage<Params extends ParamType[]> {
	(...x: Params): unknown
}