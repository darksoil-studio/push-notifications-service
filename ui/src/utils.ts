import {
	AsyncResult,
	AsyncSignal,
	AsyncState,
	JoinAsyncOptions,
	Signal,
	joinAsync,
} from '@darksoil-studio/holochain-signals';
import { GetonlyMap } from '@darksoil-studio/holochain-utils';

export function lazyLoadAndPollOrEvent<T>(
	task: () => Promise<T>,
	intervalMs: number,
	refetch: (listener: () => void) => () => void,
): AsyncSignal<T> {
	let watched = false;
	let unsubscribe: (() => void) | undefined;

	const request = () => {
		if (watched)
			task()
				.then(value => {
					if (watched)
						signal.set({
							status: 'completed',
							value,
						});
				})
				.catch(error => {
					if (watched) {
						signal.set({
							status: 'error',
							error,
						});
					}
				})
				.finally(() => {
					if (watched) {
						setTimeout(() => request(), intervalMs);
					}
				});
	};
	const signal = new AsyncState<T>(
		{ status: 'pending' },
		{
			[Signal.subtle.watched]: () => {
				watched = true;
				unsubscribe = refetch(() => request());
				request();
			},
			[Signal.subtle.unwatched]: () => {
				watched = false;
				signal.set({
					status: 'pending',
				});
				if (unsubscribe) {
					unsubscribe();
					unsubscribe = undefined;
				}
			},
		},
	);
	return signal;
}
/**
 * Create a new slice of this map that contains only the given keys
 */
export function sliceNormalMap<K, V>(
	map: GetonlyMap<K, V>,
	keys: K[],
): ReadonlyMap<K, V> {
	const newMap = new Map<K, V>();

	for (const key of keys) {
		const value = map.get(key);
		if (value) newMap.set(key, value);
	}
	return newMap;
}

/**
 * Create a new map maintaining the keys while mapping the values with the given mapping function
 */
export function mapValuesNormalMap<K, V, U>(
	map: ReadonlyMap<K, V>,
	mappingFn: (value: V, key: K) => U,
): Map<K, U> {
	const mappedMap = new Map<K, U>();

	for (const [key, value] of map.entries()) {
		mappedMap.set(key, mappingFn(value, key));
	}
	return mappedMap;
}
/**
 * Joins all the results in a HoloHashMap of `AsyncResults`
 */
export function joinAsyncNormalMap<K, T>(
	map: ReadonlyMap<K, AsyncResult<T>>,
	joinOptions?: JoinAsyncOptions,
): AsyncResult<ReadonlyMap<K, T>> {
	const resultsArray = Array.from(map.entries()).map(([key, result]) => {
		if (result.status !== 'completed') return result;
		const value = [key, result.value] as [K, T];
		return {
			status: 'completed',
			value,
		} as AsyncResult<[K, T]>;
	});
	const arrayResult = joinAsync(resultsArray, joinOptions);

	if (arrayResult.status !== 'completed') return arrayResult;

	const value = new Map<K, T>(arrayResult.value);
	return {
		status: 'completed',
		value,
	} as AsyncResult<ReadonlyMap<K, T>>;
}
