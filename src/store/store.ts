import { merge } from 'lodash'
import { Store } from "tauri-plugin-store-api";
import { SETTINGS_FILE, Cache, Settings, Wfm } from "../types";
export const store = new Store(SETTINGS_FILE)

class Persist<T> {
  constructor(public name: string, public defaults: T) { }
  async get(): Promise<T> {
    return await store.get<T>(this.name) || this.defaults
  }
  async set(key: keyof T, value: typeof this.defaults[typeof key]) {
    const currentSettings = await store.get<T>(this.name)
    // @ts-ignore
    currentSettings[key] = value
    const promise = store.set(this.name, currentSettings)
    await store.save()
    return promise
  }
  async update(newSettings: Partial<T>) {
    const currentSettings = await store.get<T>(this.name)
    const promise = store.set(this.name, merge(this.defaults, currentSettings, newSettings))
    await store.save()
    return promise
  }
  async reset() {
    const promise = store.set(this.name, this.defaults)
    await store.save()
    return promise
  }
}

export const settings = new Persist<Settings>('settings', {
  mastery_rank: 2, // Trading is unlocked at MR2
  user_email: '',
  user_password: '',
  access_token: undefined,
  budget: 0,
  current_plat: 0,
})

export const user = new Persist<Wfm.UserDto>('user', {
  banned: false,
  id: 'sdffds',
  avatar: '',
  ingame_name: '',
  locale: 'en',
  platform: 'pc',
  region: 'en',
  role: 'user',
})


export const cache = new Persist<Cache>('cache', {
  tradableItems: { createdAt: 0, items: [] },
  priceHistory: { createdAt: 0, items: [] },
})