ALTER TABLE orders
ADD COLUMN subtotal_cents BIGINT NOT NULL DEFAULT 0 CHECK (subtotal_cents >= 0),
ADD COLUMN shipping_cents BIGINT NOT NULL DEFAULT 0 CHECK (shipping_cents >= 0),
ADD COLUMN payment_method VARCHAR(30) NOT NULL DEFAULT 'bank_transfer'
    CHECK (payment_method IN ('bank_transfer', 'credit_card', 'cash_on_delivery')),
ADD COLUMN payment_status VARCHAR(20) NOT NULL DEFAULT 'paid'
    CHECK (payment_status IN ('pending', 'paid', 'refunded'));

UPDATE orders SET subtotal_cents = total_cents;

CREATE INDEX orders_payment_status_idx ON orders(payment_status, created_at DESC);
