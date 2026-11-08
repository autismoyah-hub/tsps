/** Belona buys back her own Mineral Exchange stock for 80% of its mineral price.
 * https://oldschool.runescape.wiki/w/Mining_Guild_Mineral_Exchange
 * ShopManager checks that the item is in the shop's stock and rejects instance-locked items.
 */
const SHOP = "Mining Guild Mineral Exchange";

function buysBack(request) {
  if (request.shopName === SHOP) request.buys = true;
}

function canSell(request) {
  if (request.shopName === SHOP) request.sellable = true;
}

function sellPrice(request) {
  if (request.shopName === SHOP) request.price = Math.floor(request.buyPrice * 0.8);
}

module.exports = {
  name: "Belona",
  members: true,
  register(api) {
    api.onCustomEvent("shop:buys-back", buysBack);
    api.onCustomEvent("shop:can-sell", canSell);
    api.onCustomEvent("shop:sell-price", sellPrice);
  },
};
