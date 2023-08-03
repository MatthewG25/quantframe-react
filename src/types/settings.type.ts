import { Wfm } from ".";

export interface Settings {
  mastery_rank: 2, // Trading is unlocked at MR2
  user_email: '',
  user_password: '',
  access_token: string | undefined,
  budget: 0,
  current_plat: 0,
}

export interface CacheBase {
  createdAt: number,
}

export interface TradableItemsCache extends CacheBase {
  items: Wfm.ItemDto[],
}
export interface Cache {
  tradableItems: TradableItemsCache,
  priceHistory: PriceHistoryCache,
}
export interface PriceHistoryDto {
  name: string;
  datetime: string;
  order_type: string;
  volume: number;
  min_price: number;
  max_price: number;
  range?: number;
  median: number;
  avg_price: number;
  mod_rank?: number;
  item_id: string;

  id?: string;
  open_price?: number;
  closed_price?: number;
  wa_price?: number;
  moving_avg?: number;
  donch_top?: number;
  donch_bot?: number;

}
export interface PriceHistoryCache extends CacheBase {
  items: PriceHistoryDto[],
}