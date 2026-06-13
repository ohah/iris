#import <Foundation/Foundation.h>

#if __has_include(<ReactCodegen/IrisBenchSpecs/IrisBenchSpecs.h>)
#import <ReactCodegen/IrisBenchSpecs/IrisBenchSpecs.h>
#elif __has_include(<ReactCodegen/IrisBenchSpecs.h>)
#import <ReactCodegen/IrisBenchSpecs.h>
#else
#import "IrisBenchSpecs.h"
#endif

@interface RCTIrisBenchTurboModule : NativeIrisBenchTurboModuleSpecBase <NativeIrisBenchTurboModuleSpec>

@end
