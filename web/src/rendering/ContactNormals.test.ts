import { expect, it } from 'vitest';
import { ContactNormals } from './ContactNormals';

it('draws normals from their world witnesses and clears old points after a smaller solve or reset', () => {
  const view = new ContactNormals();
  const contacts = new Float32Array([2, 1, 3, 0, 1, 0, 5, -1, 0, -2, 0, 0, 1, 12]);
  view.update(contacts);
  expect(view.visible).toBe(true);
  const vertices = view.geometry.getAttribute('position');
  expect([vertices.getX(0), vertices.getY(0), vertices.getZ(0)]).toEqual([2, 1, 3]);
  expect(vertices.getY(1)).toBeCloseTo(1.2);
  expect(vertices.getZ(3)).toBeCloseTo(-1.8);
  view.update(contacts.subarray(0, 7));
  expect(view.geometry.drawRange.count).toBe(2);
  view.update(new Float32Array());
  expect(view.visible).toBe(false);
  view.update(undefined);
  expect(view.visible).toBe(false);
});
