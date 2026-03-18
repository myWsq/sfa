export declare const packageName: string;
export declare const packageDepth: number;
export declare const packageIndex: number;
export declare const dependencyNames: string[];

export interface PackageDescription {
  packageName: string;
  packageDepth: number;
  packageIndex: number;
  dependencyNames: string[];
  schemaVersion: string;
}

export declare function describePackage(): PackageDescription;
export declare function inspectDependencies(): Array<{
  name: string;
  index: number;
  seen: boolean;
}>;
