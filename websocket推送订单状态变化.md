## 挂单

### 限价买单

```ruby
{
	symbol: "GRTUSDT",
	client_order_id: "and_62ef7ffb6aa24b639b5d016c5bc4c629",
	side: Buy,
	order_type: Limit,
	time_in_force: GTC,
	qty: 1000.0,
	price: 0.4063,
	stop_price: 0.0,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "",
	order_action: New,
	order_status: New,
	reason: "NONE",
	order_id: 763573766,
	last_qty: 0.0,
	cummulative_qty: 0.0,
	last_price: 0.0,
	fee_qty: 0.0,
	fee_quote: None,
	trade_time: 1643335143458,
	trade_id: -1,
	in_order_book: true,
	maker: false,
	order_create_time: 1643335143458,
	cummulative_vol: 0.0,
	last_vol: 0.0,
	quote_order_qty: 0.0
}
```

### 限价卖单

```ruby
{
	symbol: "GRTUSDT",
	client_order_id: "and_a5c31362bf4b409e8e3940bffecde5ac",
	side: Sell,
	order_type: Limit,
	time_in_force: GTC,
	qty: 999.0,
	price: 0.4057,
	stop_price: 0.0,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "",
	order_action: New,
	order_status: New,
	reason: "NONE",
	order_id: 763575316,
	last_qty: 0.0,
	cummulative_qty: 0.0,
	last_price: 0.0,
	fee_qty: 0.0,
	fee_quote: None,
	trade_time: 1643335219234,
	trade_id: -1,
	in_order_book: true,
	maker: false,
	order_create_time: 1643335219234,
	cummulative_vol: 0.0,
	last_vol: 0.0,
	quote_order_qty: 0.0
}
```

### 市价买单

```
{
	symbol: "LOKAUSDT",
	client_order_id: "f9iOiPGywxn6VUEqQUBALk",
	side: Buy,
	order_type: Market,
	time_in_force: GTC,
	qty: 11.6,
	price: 0.0,
	stop_price: 0.0,
	delta: None,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "",
	order_action: New,
	order_status: New,
	reason: "NONE",
	order_id: 93768149,
	last_qty: 0.0,
	cummulative_qty: 0.0,
	last_price: 0.0,
	fee_qty: 0.0,
	fee_quote: None,
	trade_time: 1652416618993,
	trade_id: -1,
	in_order_book: true,
	maker: false,
	order_create_time: 1652416618993,
	cummulative_vol: 0.0,
	last_vol: 0.0,
	quote_order_qty: 15.0
}
```

## 成交

### 完全成交

可能一次性成完成成交，可能经过多次部分成交后最后一次成交完成订单。

```ruby
# 一次性完全成交
{
	symbol: "GRTUSDT",
	client_order_id: "and_62ef7ffb6aa24b639b5d016c5bc4c629",
	side: Buy,
	order_type: Limit,
	time_in_force: GTC,
	qty: 1000.0,
	price: 0.4063,
	stop_price: 0.0,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "",
	order_action: Trade,
	order_status: Filled,
	reason: "NONE",
	order_id: 763573766,
	last_qty: 1000.0,
	cummulative_qty: 1000.0,
	last_price: 0.4063,
	fee_qty: 1.0,
	fee_quote: Some("GRT"),
	trade_time: 1643335143472,
	trade_id: 72733064,
	in_order_book: false,
	maker: true,
	order_create_time: 1643335143458,
	cummulative_vol: 406.3,
	last_vol: 406.3,
	quote_order_qty: 0.0
}

# 部分成交后，最后一次成交完成订单
{
	symbol: "GRTUSDT",
	client_order_id: "and_a5c31362bf4b409e8e3940bffecde5ac",
	side: Sell,
	order_type: Limit,
	time_in_force: GTC,
	qty: 999.0,
	price: 0.4057,
	stop_price: 0.0,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "",
	order_action: Trade,
	order_status: Filled,
	reason: "NONE",
	order_id: 763575316,
	last_qty: 718.0,
	cummulative_qty: 999.0,
	last_price: 0.4057,
	fee_qty: 0.2912926,
	fee_quote: Some("USDT"),
	trade_time: 1643335235416,
	trade_id: 72733111,
	in_order_book: false,
	maker: true,
	order_create_time: 1643335219234,
	cummulative_vol: 405.2943,
	last_vol: 291.2926,
	quote_order_qty: 0.0
}
```

### 部分成交

```ruby
{
	symbol: "GRTUSDT",
	client_order_id: "and_a5c31362bf4b409e8e3940bffecde5ac",
	side: Sell,
	order_type: Limit,
	time_in_force: GTC,
	qty: 999.0,
	price: 0.4057,
	stop_price: 0.0,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "",
	order_action: Trade,
	order_status: PartiallyFilled,
	reason: "NONE",
	order_id: 763575316,
	last_qty: 281.0,
	cummulative_qty: 281.0,
	last_price: 0.4057,
	fee_qty: 0.1140017,
	fee_quote: Some("USDT"),
	trade_time: 1643335235407,
	trade_id: 72733110,
	in_order_book: false,
	maker: true,
	order_create_time: 1643335219234,
	cummulative_vol: 114.0017,
	last_vol: 114.0017,
	quote_order_qty: 0.0
}
```

### 市价单成交

```
{
	symbol: "LOKAUSDT",
	client_order_id: "f9iOiPGywxn6VUEqQUBALk",
	side: Buy,
	order_type: Market,
	time_in_force: GTC,
	qty: 11.6,
	price: 0.0,        // 市价单成交时无价格
	stop_price: 0.0,
	delta: None,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "",
	order_action: Trade,
	order_status: Filled,
	reason: "NONE",
	order_id: 93768149,
	last_qty: 11.6,
	cummulative_qty: 11.6,
	last_price: 1.2827,
	fee_qty: 0.0116,
	fee_quote: Some("LOKA"),
	trade_time: 1652416618993,
	trade_id: 12774130,
	in_order_book: false,
	maker: false,
	order_create_time: 1652416618993,
	cummulative_vol: 14.87932,
	last_vol: 14.87932,
	quote_order_qty: 15.0
}
```



## 撤单

完全未成交时的撤单和部分成交后的撤单，可通过`cummulative_qty`或`cummulative_vol`字段来区别，完全未成交时被撤单，这两个字段都是0值，成交过被撤单，这两个字段大于0。

### 完全未成交时被撤单

```ruby
{
	symbol: "GTCUSDT",
	client_order_id: "and_fb058480cfd5456aa9ff2115ec130d2d",
	side: Buy,
	order_type: Limit,
	time_in_force: GTC,
	qty: 12.0,
	price: 8.0,
	stop_price: 0.0,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "and_ee808dc133b2439f92ba30d1eb73b652",
	order_action: Canceled,
	order_status: Canceled,
	reason: "NONE",
	order_id: 191130431,
	last_qty: 0.0,
	cummulative_qty: 0.0,
	last_price: 0.0,
	fee_qty: 0.0,
	fee_quote: None,
	trade_time: 1642762616972,
	trade_id: -1,
	in_order_book: false,
	maker: false,
	order_create_time: 1642762580413,
	cummulative_vol: 0.0,
	last_vol: 0.0,
	quote_order_qty: 0.0
}
```

### 部分成交后被撤单

```ruby
{
	symbol: "AGLDUSDT",
	client_order_id: "and_35cd92463a964990b6cf5963aa4aab04",
	side: Buy,
	order_type: Limit,
	time_in_force: GTC,
	qty: 900.0,
	price: 1.015,
	stop_price: 0.0,
	iceberg_qty: 0.0,
	order_list_id: -1,
	orig_client_order_id: "and_3c3cd105c7f147dcbaf7ce658c8e06e6",
	order_action: Canceled,
	order_status: Canceled,
	reason: "NONE",
	order_id: 34782361,
	last_qty: 0.0,
	cummulative_qty: 279.0,
	last_price: 0.0,
	fee_qty: 0.0,
	fee_quote: None,
	trade_time: 1642762858034,
	trade_id: -1,
	in_order_book: false,
	maker: false,
	order_create_time: 1642762849541,
	cummulative_vol: 283.185,
	last_vol: 0.0,
	quote_order_qty: 0.0
}
```



